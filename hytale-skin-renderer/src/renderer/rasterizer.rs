//! Core triangle rasterization with depth buffer and texture sampling
//!
//! Provides per-pixel triangle rendering with:
//! - Depth testing via z-buffer
//! - UV interpolation using barycentric coordinates
//! - Texture sampling with optional bilinear filtering
//! - Complex tinting logic for body parts

use crate::error::Result;
use crate::texture::{
    sample_face_texture, sample_face_texture_bilinear, sample_face_texture_tinted,
    sample_face_texture_tinted_bilinear, Texture, TintGradient,
};
use image::RgbaImage;

use super::config::{RenderConfig, TintConfig};
use super::math::barycentric_coords;

/// Small depth bias to prevent Z-fighting between coplanar surfaces.
/// A surface must be this much closer than the current depth to overwrite it.
const DEPTH_BIAS: f32 = 0.001;

/// Render a triangle with depth buffer, texture sampling, and optional tinting
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_triangle_tinted(
    image: &mut RgbaImage,
    depth_buffer: &mut [f32],
    output_width: u32,
    vertices: &[(f32, f32, f32)],
    uvs: &[(f32, f32)],
    texture: &Texture,
    uv_face: &crate::models::UvFace,
    face_width: f32,
    face_height: f32,
    node_name: Option<&str>,
    tint_config: Option<&TintConfig>,
    config: RenderConfig,
    specific_tint: Option<&TintGradient>,
    face_normal: glam::Vec3,
) -> Result<()> {
    if vertices.len() != 3 || uvs.len() != 3 {
        return Ok(());
    }

    let (x0, y0, z0) = vertices[0];
    let (x1, y1, z1) = vertices[1];
    let (x2, y2, z2) = vertices[2];

    let (uv0_u, uv0_v) = uvs[0];
    let (uv1_u, uv1_v) = uvs[1];
    let (uv2_u, uv2_v) = uvs[2];

    let min_x = x0.min(x1).min(x2).max(0.0) as u32;
    let max_x = x0.max(x1).max(x2).min(image.width() as f32) as u32;
    let min_y = y0.min(y1).min(y2).max(0.0) as u32;
    let max_y = y0.max(y1).max(y2).min(image.height() as f32) as u32;

    for y in min_y..=max_y.min(image.height() - 1) {
        for x in min_x..=max_x.min(image.width() - 1) {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let (bary_u, bary_v, bary_w) = barycentric_coords(px, py, x0, y0, x1, y1, x2, y2);

            if bary_u >= 0.0 && bary_v >= 0.0 && bary_w >= 0.0 {
                // Interpolate depth and check buffer
                let depth = bary_w * z0 + bary_v * z1 + bary_u * z2;
                let buffer_index = (y * output_width + x) as usize;

                if depth < depth_buffer[buffer_index] - DEPTH_BIAS {
                    // Interpolate UV coordinates (vertex weights: v0=w, v1=v, v2=u)
                    let tex_u = bary_w * uv0_u + bary_v * uv1_u + bary_u * uv2_u;
                    let tex_v = bary_w * uv0_v + bary_v * uv1_v + bary_u * uv2_v;

                    let pixel = if let Some(tint) = specific_tint {
                        if config.bilinear_filtering {
                            sample_face_texture_tinted_bilinear(
                                texture,
                                uv_face,
                                face_width,
                                face_height,
                                tex_u,
                                tex_v,
                                tint,
                            )
                        } else {
                            sample_face_texture_tinted(
                                texture,
                                uv_face,
                                face_width,
                                face_height,
                                tex_u,
                                tex_v,
                                tint,
                            )
                        }
                    } else if let Some(tc) = tint_config {
                        if let Some(name) = node_name {
                            if let Some(gradient) = tc.get_tint_for_node(name) {
                                if config.bilinear_filtering {
                                    sample_face_texture_tinted_bilinear(
                                        texture,
                                        uv_face,
                                        face_width,
                                        face_height,
                                        tex_u,
                                        tex_v,
                                        gradient,
                                    )
                                } else {
                                    sample_face_texture_tinted(
                                        texture,
                                        uv_face,
                                        face_width,
                                        face_height,
                                        tex_u,
                                        tex_v,
                                        gradient,
                                    )
                                }
                            } else {
                                if config.bilinear_filtering {
                                    sample_face_texture_bilinear(
                                        texture,
                                        uv_face,
                                        face_width,
                                        face_height,
                                        tex_u,
                                        tex_v,
                                    )
                                } else {
                                    sample_face_texture(
                                        texture,
                                        uv_face,
                                        face_width,
                                        face_height,
                                        tex_u,
                                        tex_v,
                                    )
                                }
                            }
                        } else if config.bilinear_filtering {
                            sample_face_texture_bilinear(
                                texture,
                                uv_face,
                                face_width,
                                face_height,
                                tex_u,
                                tex_v,
                            )
                        } else {
                            sample_face_texture(
                                texture,
                                uv_face,
                                face_width,
                                face_height,
                                tex_u,
                                tex_v,
                            )
                        }
                    } else if config.bilinear_filtering {
                        sample_face_texture_bilinear(
                            texture,
                            uv_face,
                            face_width,
                            face_height,
                            tex_u,
                            tex_v,
                        )
                    } else {
                        sample_face_texture(texture, uv_face, face_width, face_height, tex_u, tex_v)
                    };

                    // Apply lighting if enabled (after tinting, before alpha check)
                    let pixel = if config.light_config.enabled {
                        let n_dot_l = face_normal
                            .dot(config.light_config.light_direction)
                            .max(0.0);
                        let lighting = (config.light_config.ambient
                            + config.light_config.diffuse * n_dot_l)
                            .min(1.0);

                        // Gamma-correct shading
                        // Convert sRGB to linear, apply lighting, then back to sRGB
                        let apply_gamma = |c: u8| -> u8 {
                            let linear = (c as f32 / 255.0).powf(2.2) * lighting;
                            (linear.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8
                        };

                        image::Rgba([
                            apply_gamma(pixel[0]),
                            apply_gamma(pixel[1]),
                            apply_gamma(pixel[2]),
                            pixel[3], // Alpha unchanged
                        ])
                    } else {
                        pixel
                    };

                    if pixel[3] > 0 {
                        depth_buffer[buffer_index] = depth;
                        image.put_pixel(x, y, pixel);
                    }
                }
            }
        }
    }

    Ok(())
}
