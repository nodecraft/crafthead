//! 3D-to-2D rendering pipeline with depth sorting
//!
//! This module provides a complete rendering pipeline for converting 3D geometry
//! into 2D images with proper depth handling, texture mapping, and tinting support.

mod clip;
mod config;
mod debug;
mod face;
mod math;
mod postprocess;
mod rasterizer;

// Re-export public API
pub use config::{LightConfig, RenderConfig, TintConfig};

use crate::camera::CameraProjection;
use crate::error::Result;
use crate::geometry::Face;
use crate::models::Vector3;
use crate::texture::Texture;
use image::RgbaImage;
use std::sync::Arc;

use debug::{render_face_to_image_debug, RenderFace};
use face::render_face_to_image_tinted;
use postprocess::apply_blur;

/// Render a scene to a 2D image
///
/// Accepts faces with optional shape information for texture layout.
/// If shape is None, default UV coordinates are used.
pub fn render_scene(
    faces: &[RenderableFace],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
) -> Result<RgbaImage> {
    render_scene_internal(
        faces,
        texture,
        camera,
        output_width,
        output_height,
        false,
        None,
        RenderConfig::default(),
    )
}

/// A face to be rendered with all associated metadata
#[derive(Clone, Debug)]
pub struct RenderableFace {
    pub face: Face,
    pub transform: glam::Mat4,
    pub shape: Option<crate::models::Shape>,
    pub node_name: Option<String>,
    pub texture: Option<Arc<Texture>>,
    pub tint: Option<Arc<crate::texture::TintGradient>>,
}

/// Render a scene to a 2D image with tinting applied
///
/// Accepts faces with shape information, node names for tint mapping,
/// and a tint configuration.
pub fn render_scene_tinted(
    faces: &[RenderableFace],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
    tint_config: &TintConfig,
) -> Result<RgbaImage> {
    render_scene_tinted_with_config(
        faces,
        texture,
        camera,
        output_width,
        output_height,
        tint_config,
        RenderConfig::default(),
    )
}

/// Render a scene to a 2D image with tinting and configurable filtering
pub fn render_scene_tinted_with_config(
    faces: &[RenderableFace],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
    tint_config: &TintConfig,
    config: RenderConfig,
) -> Result<RgbaImage> {
    let mut image = render_scene_internal(
        faces,
        texture,
        camera,
        output_width,
        output_height,
        false,
        Some(tint_config),
        config,
    )?;

    // Apply post-processing blur if requested
    if config.blur_amount > 0.0 {
        apply_blur(&mut image, config.blur_amount);
    }

    Ok(image)
}

/// Render a scene to a 2D image with optional shape for texture layout (deprecated - use render_scene with per-face shapes)
#[deprecated(note = "Use render_scene with per-face shape information instead")]
pub fn render_scene_with_shape(
    faces: &[(Face, glam::Mat4)],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
    shape: Option<&crate::models::Shape>,
) -> Result<RgbaImage> {
    // Convert old API to new API
    let faces_as_structs: Vec<RenderableFace> = faces
        .iter()
        .map(|(face, transform)| RenderableFace {
            face: face.clone(),
            transform: *transform,
            shape: shape.cloned(),
            node_name: None,
            texture: None,
            tint: None,
        })
        .collect();
    render_scene_with_shape_debug(
        &faces_as_structs,
        texture,
        camera,
        output_width,
        output_height,
        false,
    )
}

/// Render a scene to a 2D image with optional shape and debug mode
/// Debug mode colors faces by direction: front=red, back=green, left=blue, right=yellow, top=cyan, bottom=magenta
pub fn render_scene_with_shape_debug(
    faces: &[RenderableFace],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
    debug_mode: bool,
) -> Result<RgbaImage> {
    render_scene_internal(
        faces,
        texture,
        camera,
        output_width,
        output_height,
        debug_mode,
        None,
        RenderConfig::default(),
    )
}

/// Internal render function supporting both tinted and non-tinted rendering
///
/// Uses a per-pixel depth buffer (z-buffer) for accurate depth testing.
/// Faces can be rendered in any order - the z-buffer ensures correct visibility.
fn render_scene_internal(
    faces: &[RenderableFace],
    texture: &Texture,
    camera: &dyn CameraProjection,
    output_width: u32,
    output_height: u32,
    debug_mode: bool,
    tint_config: Option<&TintConfig>,
    config: RenderConfig,
) -> Result<RgbaImage> {
    let mut image = RgbaImage::new(output_width, output_height);

    // Create depth buffer initialized to maximum depth (far plane)
    // Using a 1D vector with 2D indexing for better cache performance
    let mut depth_buffer = vec![f32::MAX; (output_width * output_height) as usize];

    // Project all faces to screen space
    let mut render_faces: Vec<RenderFace> = Vec::new();

    // Track unique shapes to assign part indices for debug coloring
    let mut shape_to_index: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    let mut next_part_index = 0usize;

    // Get view-projection matrix once for all faces
    let vp_matrix = camera.view_projection_matrix(output_width, output_height);

    for render_face in faces {
        let (face, _transform, shape, node_name, specific_texture, specific_tint) = (
            &render_face.face,
            &render_face.transform,
            &render_face.shape,
            &render_face.node_name,
            &render_face.texture,
            &render_face.tint,
        );
        // Determine part index based on shape pointer (each unique shape = different body part)
        let part_index = if let Some(ref s) = shape {
            let shape_ptr = s as *const _ as usize;
            *shape_to_index.entry(shape_ptr).or_insert_with(|| {
                let idx = next_part_index;
                next_part_index += 1;
                idx
            })
        } else {
            0
        };

        // Clip face to frustum (returns clipped vertices in clip space)
        if let Some(clipped_vertices) = clip::clip_face_to_frustum(face, &vp_matrix) {
            // Extract face normal from first vertex for lighting
            let face_normal = clipped_vertices[0].normal;

            // Project clipped vertices to screen space
            let mut screen_vertices = Vec::new();
            let mut face_vertices_world = Vec::new();

            for clip_vertex in &clipped_vertices {
                // Perform perspective divide
                let ndc = glam::Vec3::new(
                    clip_vertex.clip_pos.x / clip_vertex.clip_pos.w,
                    clip_vertex.clip_pos.y / clip_vertex.clip_pos.w,
                    clip_vertex.clip_pos.z / clip_vertex.clip_pos.w,
                );

                // Convert to screen space
                let screen_x = (ndc.x + 1.0) * 0.5 * output_width as f32;
                let screen_y = (1.0 - ndc.y) * 0.5 * output_height as f32;

                // Calculate view-space depth for z-buffer
                let world_pos_vec3 = Vector3 {
                    x: clip_vertex.world_pos.x,
                    y: clip_vertex.world_pos.y,
                    z: clip_vertex.world_pos.z,
                };
                let view_depth = camera.calculate_depth(world_pos_vec3);

                screen_vertices.push((screen_x, screen_y, view_depth));

                // Rebuild face vertex list for rendering
                face_vertices_world.push(crate::geometry::Vertex {
                    position: clip_vertex.world_pos,
                    normal: glam::Vec3::ZERO, // Not used in rendering
                    uv: clip_vertex.uv,
                });
            }

            // Only render if we have at least 3 vertices (triangle)
            if screen_vertices.len() >= 3 {
                // Backface culling (skip for double-sided shapes)
                let is_double_sided = shape.as_ref().map_or(false, |s| s.double_sided);

                if !is_double_sided {
                    // Check if winding is flipped due to negative stretch
                    // Odd number of negative axes = flipped winding
                    let winding_flipped = shape.as_ref().map_or(false, |s| {
                        let neg_count = [s.stretch.x < 0.0, s.stretch.y < 0.0, s.stretch.z < 0.0]
                            .iter()
                            .filter(|&&b| b)
                            .count();
                        neg_count % 2 == 1 // Odd count = flipped
                    });

                    let (x0, y0, _) = screen_vertices[0];
                    let (x1, y1, _) = screen_vertices[1];
                    let (x2, y2, _) = screen_vertices[2];

                    // Signed area via cross product (2D)
                    // Note: Screen Y points down (1.0 - ndc.y), so winding is reversed:
                    // Positive signed area = CW = back-facing (cull)
                    // Negative signed area = CCW = front-facing (keep)
                    let signed_area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);

                    // If winding is flipped by negative stretch, invert the check
                    let is_backfacing = if winding_flipped {
                        signed_area < 0.0 // Flipped: negative = back-facing
                    } else {
                        signed_area > 0.0 // Normal: positive = back-facing
                    };

                    if is_backfacing {
                        continue; // Skip back-facing triangle
                    }
                }

                let clipped_face = Face {
                    vertices: face_vertices_world,
                    texture_face: face.texture_face.clone(),
                };

                render_faces.push(RenderFace {
                    screen_vertices,
                    texture_face: face.texture_face.clone(),
                    face_data: clipped_face,
                    shape: shape.clone(),
                    part_index,
                    node_name: node_name.clone(),
                    texture: specific_texture.clone(),
                    tint_gradient: specific_tint.clone(),
                    normal: face_normal,
                });
            }
        }
    }

    // Render each face
    for render_face in &render_faces {
        if debug_mode {
            render_face_to_image_debug(&mut image, &mut depth_buffer, output_width, render_face)?;
        } else {
            render_face_to_image_tinted(
                &mut image,
                &mut depth_buffer,
                output_width,
                render_face,
                texture,
                render_face.shape.as_ref(),
                tint_config,
                config,
            )?;
        }
    }

    Ok(image)
}
