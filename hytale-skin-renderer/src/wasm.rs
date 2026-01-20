//! WASM bindings for browser/Cloudflare Workers integration
//!
//! These functions provide a JavaScript-friendly API for rendering Hytale
//! characters from pre-loaded model, animation, and texture data.

use wasm_bindgen::prelude::*;

use crate::{animation, camera, geometry, models, output, renderer, scene, texture};

/// Render a Hytale character to PNG bytes
///
/// # Arguments
/// * `model_json` - BlockyModel JSON string (Player.blockymodel contents)
/// * `animation_json` - BlockyAnimation JSON string (Idle.blockyanim contents)
/// * `texture_bytes` - PNG texture data as bytes
/// * `view_type` - Camera preset: "headshot", "isometric_head", "player_bust", "full_body_front"
/// * `width` - Output image width
/// * `height` - Output image height
///
/// # Returns
/// PNG image bytes on success, or an error string
#[wasm_bindgen]
pub fn render_hytale(
	model_json: &str,
	animation_json: &str,
	texture_bytes: &[u8],
	view_type: &str,
	width: u32,
	height: u32,
) -> Result<Vec<u8>, JsValue> {
	// Parse model and animation from JSON
	let model = models::parse_blockymodel(model_json)
		.map_err(|e| JsValue::from_str(&format!("Model parse error: {}", e)))?;
	let animation = models::parse_blockyanim(animation_json)
		.map_err(|e| JsValue::from_str(&format!("Animation parse error: {}", e)))?;

	// Load texture from bytes
	let tex = texture::Texture::from_bytes(texture_bytes)
		.map_err(|e| JsValue::from_str(&format!("Texture load error: {}", e)))?;

	// Sample animation at frame 0 for idle pose
	let pose = animation::sample_animation(&animation, 0.0);

	// Create scene graph with pose applied
	let scene_graph = scene::SceneGraph::from_blockymodel_with_pose(&model, &pose, None)
		.map_err(|e| JsValue::from_str(&format!("Scene graph error: {}", e)))?;

	// Collect visible shapes and generate geometry
	let visible_shapes = scene_graph.get_visible_shapes();
	let mut faces = Vec::new();
	for (node, transform) in &visible_shapes {
		if let Some(ref shape) = node.shape {
			let geom = geometry::generate_geometry(shape, *transform);
			for face in geom {
				faces.push(renderer::RenderableFace {
					face,
					transform: *transform,
					shape: Some(shape.clone()),
					node_name: Some(node.name.clone()),
					texture: None,
					tint: None,
				});
			}
		}
	}

	// Select camera based on view type
	let cam: Box<dyn camera::CameraProjection> = match view_type {
		"headshot" => Box::new(camera::PerspectiveCamera::headshot()),
		"isometric_head" => Box::new(camera::PerspectiveCamera::isometric_head()),
		"player_bust" => Box::new(camera::PerspectiveCamera::player_bust()),
		"full_body_front" => Box::new(camera::Camera::full_body_front()),
		"front_right" => Box::new(camera::Camera::front_right_view()),
		"back_right" => Box::new(camera::Camera::back_right_view()),
		_ => Box::new(camera::PerspectiveCamera::headshot()),
	};

	// Render scene
	let image = renderer::render_scene(&faces, &tex, cam.as_ref(), width, height)
		.map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

	// Export to PNG bytes
	output::export_png_bytes(&image).map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
}

#[derive(serde::Deserialize)]
pub struct Cosmetic {
	pub model_json: String,
	pub texture_bytes: Vec<u8>,
	pub tint_colors: Option<Vec<String>>,
	pub tint_texture_bytes: Option<Vec<u8>>,
}

/// Render a Hytale character with cosmetics to PNG bytes
#[wasm_bindgen]
pub fn render_hytale_with_cosmetics(
	model_json: &str,
	animation_json: &str,
	base_texture_bytes: &[u8],
	cosmetics_js: JsValue,
	base_tint_colors: Option<Vec<String>>,
	base_tint_texture_bytes: Option<Vec<u8>>,
	view_type: &str,
	width: u32,
	height: u32,
) -> Result<Vec<u8>, JsValue> {
	// Deserialize cosmetics
	let cosmetics: Vec<Cosmetic> = serde_wasm_bindgen::from_value(cosmetics_js)?;

	// Parse model and animation
	let mut model = models::parse_blockymodel(model_json)
		.map_err(|e| JsValue::from_str(&format!("Model parse error: {}", e)))?;
	let animation = models::parse_blockyanim(animation_json)
		.map_err(|e| JsValue::from_str(&format!("Animation parse error: {}", e)))?;

	// Load textures
	// Index 0 is base texture
	let base_texture = texture::Texture::from_bytes(base_texture_bytes)
		.map_err(|e| JsValue::from_str(&format!("Base texture load error: {}", e)))?;

	let mut textures = vec![std::sync::Arc::new(base_texture)];
	// Create base tint if provided
	let base_tint = if let Some(texture_bytes) = base_tint_texture_bytes {
		// Prioritize texture tint
		texture::TintGradient::from_bytes(&texture_bytes)
			.map(std::sync::Arc::new)
			.ok()
	} else if let Some(colors) = base_tint_colors {
		Some(std::sync::Arc::new(texture::TintGradient::from_hex_colors(
			&colors,
		)))
	} else {
		None
	};
	let mut tints = vec![base_tint];

	// Parse cosmetic models and textures
	let mut cosmetic_graphs = Vec::new();

	for (i, cosmetic) in cosmetics.iter().enumerate() {
		let cosmetic_model = models::parse_blockymodel(&cosmetic.model_json)
			.map_err(|e| JsValue::from_str(&format!("Cosmetic {} model parse error: {}", i, e)))?;

		let cosmetic_texture = texture::Texture::from_bytes(&cosmetic.texture_bytes)
			.map_err(|e| JsValue::from_str(&format!("Cosmetic {} texture load error: {}", i, e)))?;

		textures.push(std::sync::Arc::new(cosmetic_texture));

		// Create tint gradient if provided
		let cosmetic_tint = if let Some(texture_bytes) = &cosmetic.tint_texture_bytes {
			texture::TintGradient::from_bytes(texture_bytes)
				.map(std::sync::Arc::new)
				.ok()
		} else if let Some(colors) = &cosmetic.tint_colors {
			Some(std::sync::Arc::new(texture::TintGradient::from_hex_colors(
				colors,
			)))
		} else {
			None
		};
		tints.push(cosmetic_tint);

		let graph = scene::SceneGraph::from_blockymodel(&cosmetic_model)
			.map_err(|e| JsValue::from_str(&format!("Cosmetic {} graph error: {}", i, e)))?;

		cosmetic_graphs.push(graph);
	}

	// Sample animation
	let pose = animation::sample_animation(&animation, 0.0);

	// Create base scene graph
	let mut scene_graph = scene::SceneGraph::from_blockymodel_with_pose(&model, &pose, None)
		.map_err(|e| JsValue::from_str(&format!("Scene graph error: {}", e)))?;

	// Merge cosmetics (starting from texture index 1)
	for (i, graph) in cosmetic_graphs.into_iter().enumerate() {
		scene_graph.merge_graph(graph, i + 1);
	}

	// Collect visible shapes and generate geometry
	let visible_shapes = scene_graph.get_visible_shapes();
	let mut faces = Vec::new();
	for (node, transform) in &visible_shapes {
		if let Some(ref shape) = node.shape {
			let geom = geometry::generate_geometry(shape, *transform);
			for face in geom {
				let specific_texture = node.texture_id.map(|id| {
					if id < textures.len() {
						textures[id].clone()
					} else {
						textures[0].clone()
					}
				});

				faces.push(renderer::RenderableFace {
					face,
					transform: *transform,
					shape: Some(shape.clone()),
					node_name: Some(node.name.clone()),
					texture: specific_texture,
					tint: if let Some(id) = node.texture_id {
						if id < tints.len() {
							tints[id].clone()
						} else {
							None
						}
					} else {
						tints[0].clone()
					},
				});
			}
		}
	}

	// Select camera based on view type
	let cam: Box<dyn camera::CameraProjection> = match view_type {
		"headshot" => Box::new(camera::PerspectiveCamera::headshot()),
		"isometric_head" => Box::new(camera::PerspectiveCamera::isometric_head()),
		"player_bust" => Box::new(camera::PerspectiveCamera::player_bust()),
		"full_body_front" => Box::new(camera::Camera::full_body_front()),
		"front_right" => Box::new(camera::Camera::front_right_view()),
		"back_right" => Box::new(camera::Camera::back_right_view()),
		_ => Box::new(camera::PerspectiveCamera::headshot()),
	};

	// Render scene
	// Pass base texture as default (though specific textures will override it)
	let image = renderer::render_scene(&faces, &textures[0], cam.as_ref(), width, height)
		.map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

	// Export to PNG bytes
	output::export_png_bytes(&image).map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
}

/// Get available view types as a JSON array
#[wasm_bindgen]
pub fn get_available_view_types() -> String {
	r#"["headshot", "isometric_head", "player_bust", "full_body_front", "front_right", "back_right"]"#
        .to_string()
}
