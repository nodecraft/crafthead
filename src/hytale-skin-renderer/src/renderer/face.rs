//! Face-to-triangle decomposition and rendering
//!
//! Handles converting face quads to triangles and managing texture mapping.

use crate::error::Result;
use crate::models::Vector3;
use crate::texture::Texture;
use image::RgbaImage;

use super::config::{RenderConfig, TintConfig};
use super::debug::RenderFace;
use super::rasterizer::{render_triangle_tinted, FaceSampler, ShadeCache};

/// Render a face to the image with optional tinting
///
/// Resolves everything that is constant across the face (texture layout, tint, light level)
/// once, then converts the face into triangles for the rasterizer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_face_to_image_tinted(
	image: &mut RgbaImage,
	depth_buffer: &mut [f32],
	output_width: u32,
	render_face: &RenderFace,
	texture: &Texture,
	tint_config: Option<&TintConfig>,
	config: RenderConfig,
	shades: &mut ShadeCache,
) -> Result<()> {
	let source = render_face.source;
	let texture_face = source.face.texture_face.as_str();

	// Skip rendering faces that have no texture layout defined
	// This prevents garbage rendering on shapes that only define a subset of faces
	// (e.g., head accessories with only front/right layouts)
	let Some(shape) = source.shape.as_ref() else {
		return Ok(());
	};
	let uv_face = match texture_face {
		"front" => shape.texture_layout.front.as_ref(),
		"back" => shape.texture_layout.back.as_ref(),
		"left" => shape.texture_layout.left.as_ref(),
		"right" => shape.texture_layout.right.as_ref(),
		"top" => shape.texture_layout.top.as_ref(),
		"bottom" => shape.texture_layout.bottom.as_ref(),
		_ => None,
	};
	let Some(uv_face) = uv_face else {
		return Ok(());
	};

	let size = shape.settings.size.unwrap_or(Vector3 {
		x: 1.0,
		y: 1.0,
		z: 1.0,
	});
	let (face_width, face_height) = match texture_face {
		"front" | "back" => (size.x, size.y),
		"left" | "right" => (size.z, size.y),
		"top" | "bottom" => (size.x, size.z),
		_ => (1.0, 1.0),
	};

	// With selective greyscale tinting, we can safely apply tint_config to all textures.
	// The apply_tint function will automatically preserve pre-colored pixels and only
	// tint greyscale areas, so we don't need node-name-based heuristics anymore.
	let tint = match (
		source.tint.as_deref(),
		tint_config,
		source.node_name.as_deref(),
	) {
		(Some(specific_tint), _, _) => Some(specific_tint),
		(None, Some(tint_config), Some(node_name)) => tint_config.get_tint_for_node(node_name),
		_ => None,
	};

	let sampler = FaceSampler {
		texture: source.texture.as_deref().unwrap_or(texture),
		uv_face,
		face_width,
		face_height,
		tint,
		bilinear_filtering: config.bilinear_filtering,
	};

	let mut shade = if config.light_config.enabled {
		let n_dot_l = render_face
			.normal
			.dot(config.light_config.light_direction)
			.max(0.0);
		let lighting =
			(config.light_config.ambient + config.light_config.diffuse * n_dot_l).min(1.0);
		Some(shades.level(lighting))
	} else {
		None
	};

	let vertices = &render_face.screen_vertices;
	let uvs = &render_face.uvs;

	// Render generically as a triangle fan
	// This handles triangles, quads, and clipped polygons (pentagons, hexagons, etc.)
	// Triangle 0: (v0, v1, v2)
	// Triangle 1: (v0, v2, v3)
	// ...
	for i in 1..vertices.len().saturating_sub(1) {
		render_triangle_tinted(
			image,
			depth_buffer,
			output_width,
			[vertices[0], vertices[i], vertices[i + 1]],
			[uvs[0], uvs[i], uvs[i + 1]],
			&sampler,
			shade.as_deref_mut(),
		);
	}

	Ok(())
}
