use crate::error::Result;
use glam::Vec3;
use image::RgbaImage;

use super::math::barycentric_coords;

/// A renderable face with screen coordinates (used internally for rendering)
#[derive(Clone)]
pub(crate) struct RenderFace {
	pub screen_vertices: Vec<(f32, f32, f32)>, // (x, y, depth)
	pub texture_face: String,
	pub face_data: crate::geometry::Face,
	pub shape: Option<crate::models::Shape>, // Store shape for texture layout access
	pub part_index: usize,                   // Index to differentiate body parts for debug colors
	pub node_name: Option<String>,           // Node name for tint mapping
	pub texture: Option<std::sync::Arc<crate::texture::Texture>>, // Optional specific texture
	pub tint_gradient: Option<std::sync::Arc<crate::texture::TintGradient>>, // Optional specific tint gradient
	pub normal: Vec3, // Face normal for lighting
}

pub(crate) fn get_debug_color(face_name: &str, part_index: usize) -> image::Rgba<u8> {
	// Base colors for each face direction
	let base_color: (f32, f32, f32) = match face_name {
		"front" => (255.0, 0.0, 0.0),    // Red
		"back" => (0.0, 255.0, 0.0),     // Green
		"left" => (0.0, 0.0, 255.0),     // Blue
		"right" => (255.0, 255.0, 0.0),  // Yellow
		"top" => (0.0, 255.0, 255.0),    // Cyan
		"bottom" => (255.0, 0.0, 255.0), // Magenta
		_ => (128.0, 128.0, 128.0),      // Gray for unknown
	};

	// Vary brightness based on part index (cycle through 5 brightness levels)
	// This creates distinct shades: 100%, 85%, 70%, 55%, 40% brightness
	let brightness_levels = [1.0, 0.85, 0.70, 0.55, 0.80];
	let brightness = brightness_levels[part_index % brightness_levels.len()];

	// Apply brightness and clamp
	let r = (base_color.0 * brightness).min(255.0) as u8;
	let g = (base_color.1 * brightness).min(255.0) as u8;
	let b = (base_color.2 * brightness).min(255.0) as u8;

	image::Rgba([r, g, b, 255])
}

/// Render face with debug colors (no texture, just colored faces)
pub(crate) fn render_face_to_image_debug(
	image: &mut RgbaImage,
	depth_buffer: &mut [f32],
	output_width: u32,
	render_face: &RenderFace,
) -> Result<()> {
	if render_face.screen_vertices.len() < 3 {
		return Ok(()); // Need at least 3 vertices for a triangle
	}

	let debug_color = get_debug_color(&render_face.texture_face, render_face.part_index);

	// Render as triangles (quad = 2 triangles)
	let vertices = &render_face.screen_vertices;

	if vertices.len() == 4 {
		// Quad: render as two triangles
		render_triangle_debug(
			image,
			depth_buffer,
			output_width,
			&[vertices[0], vertices[1], vertices[2]],
			debug_color,
		)?;
		render_triangle_debug(
			image,
			depth_buffer,
			output_width,
			&[vertices[0], vertices[2], vertices[3]],
			debug_color,
		)?;
	} else if vertices.len() >= 3 {
		// Triangle or polygon: render first triangle
		render_triangle_debug(
			image,
			depth_buffer,
			output_width,
			&[vertices[0], vertices[1], vertices[2]],
			debug_color,
		)?;
	}

	Ok(())
}

/// Render a triangle with a solid debug color using the depth buffer
pub(crate) fn render_triangle_debug(
	image: &mut RgbaImage,
	depth_buffer: &mut [f32],
	output_width: u32,
	vertices: &[(f32, f32, f32)],
	color: image::Rgba<u8>,
) -> Result<()> {
	if vertices.len() != 3 {
		return Ok(());
	}

	let (x0, y0, z0) = vertices[0];
	let (x1, y1, z1) = vertices[1];
	let (x2, y2, z2) = vertices[2];

	// Find bounding box
	let min_x = x0.min(x1).min(x2).max(0.0) as u32;
	let max_x = x0.max(x1).max(x2).min(image.width() as f32) as u32;
	let min_y = y0.min(y1).min(y2).max(0.0) as u32;
	let max_y = y0.max(y1).max(y2).min(image.height() as f32) as u32;

	// Rasterize triangle
	for y in min_y..=max_y.min(image.height() - 1) {
		for x in min_x..=max_x.min(image.width() - 1) {
			let px = x as f32 + 0.5;
			let py = y as f32 + 0.5;

			// Barycentric coordinates
			let (bary_u, bary_v, bary_w) = barycentric_coords(px, py, x0, y0, x1, y1, x2, y2);

			if bary_u >= 0.0 && bary_v >= 0.0 && bary_w >= 0.0 {
				// Interpolate depth using barycentric coordinates
				// Note: barycentric_coords returns (u, v, w) where u is for vertex 2, v is for vertex 1, w is for vertex 0
				let depth = bary_w * z0 + bary_v * z1 + bary_u * z2;

				// Depth test: only render if closer than existing depth
				let buffer_index = (y * output_width + x) as usize;
				if depth < depth_buffer[buffer_index] {
					depth_buffer[buffer_index] = depth;
					image.put_pixel(x, y, color);
				}
			}
		}
	}

	Ok(())
}
