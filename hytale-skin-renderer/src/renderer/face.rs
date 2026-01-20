//! Face-to-triangle decomposition and rendering
//!
//! Handles converting face quads to triangles and managing texture mapping.

use crate::error::Result;
use crate::models::Vector3;
use crate::texture::Texture;
use image::RgbaImage;

use super::config::{RenderConfig, TintConfig};
use super::debug::RenderFace;
use super::rasterizer::render_triangle_tinted;

/// Render a face to the image with optional tinting
///
/// Converts face quads into triangles and calls the rasterizer for each triangle.
pub(crate) fn render_face_to_image_tinted(
    image: &mut RgbaImage,
    depth_buffer: &mut [f32],
    output_width: u32,
    render_face: &RenderFace,
    texture: &Texture,
    shape: Option<&crate::models::Shape>,
    tint_config: Option<&TintConfig>,
    config: RenderConfig,
) -> Result<()> {
    // Get texture face mapping from shape, or use default
    let uv_face = if let Some(s) = shape {
        match render_face.texture_face.as_str() {
            "front" => s.texture_layout.front.as_ref(),
            "back" => s.texture_layout.back.as_ref(),
            "left" => s.texture_layout.left.as_ref(),
            "right" => s.texture_layout.right.as_ref(),
            "top" => s.texture_layout.top.as_ref(),
            "bottom" => s.texture_layout.bottom.as_ref(),
            _ => None,
        }
    } else {
        None
    };

    // Skip rendering faces that have no texture layout defined
    // This prevents garbage rendering on shapes that only define a subset of faces
    // (e.g., head accessories with only front/right layouts)
    let uv_face = match uv_face {
        Some(face) => face,
        None => return Ok(()), // Skip this face entirely
    };

    let (face_width, face_height) = if let Some(s) = shape {
        let size = s.settings.size.unwrap_or(Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        });
        match render_face.texture_face.as_str() {
            "front" | "back" => (size.x, size.y),
            "left" | "right" => (size.z, size.y),
            "top" | "bottom" => (size.x, size.z),
            _ => (1.0, 1.0),
        }
    } else {
        (1.0, 1.0)
    };

    let texture = render_face.texture.as_deref().unwrap_or(texture);

    // With selective greyscale tinting, we can safely apply tint_config to all textures.
    // The apply_tint function will automatically preserve pre-colored pixels and only
    // tint greyscale areas, so we don't need node-name-based heuristics anymore.
    let effective_tint_config = tint_config;

    let vertices = &render_face.screen_vertices;
    let face_uvs: Vec<(f32, f32)> = render_face
        .face_data
        .vertices
        .iter()
        .map(|v| v.uv)
        .collect();

    if vertices.len() >= 3 && face_uvs.len() >= 3 {
        // Render generically as a triangle fan
        // This handles triangles, quads, and clipped polygons (pentagons, hexagons, etc.)
        // Triangle 0: (v0, v1, v2)
        // Triangle 1: (v0, v2, v3)
        // ...
        for i in 1..(vertices.len() - 1) {
            render_triangle_tinted(
                image,
                depth_buffer,
                output_width,
                &[vertices[0], vertices[i], vertices[i + 1]],
                &[face_uvs[0], face_uvs[i], face_uvs[i + 1]],
                texture,
                uv_face,
                face_width,
                face_height,
                render_face.node_name.as_deref(),
                effective_tint_config,
                config,
                render_face.tint_gradient.as_deref(),
                render_face.normal,
            )?;
        }
    }

    Ok(())
}
