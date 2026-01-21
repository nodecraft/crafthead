//! WASM bindings for browser/Cloudflare Workers integration
//!
//! These functions provide a JavaScript-friendly API for rendering Hytale
//! characters from pre-loaded model, animation, and texture data.

use wasm_bindgen::prelude::*;

use crate::asset_provider::{AssetProvider, MemoryAssetProvider};
use crate::{camera, models, output, render_pipeline, renderer, skin, texture};

/// Render a Hytale character with full cosmetic support
///
/// # Arguments
/// * `model_json` - BlockyModel JSON string (Player.blockymodel contents)
/// * `animation_json` - BlockyAnimation JSON string (Idle.blockyanim contents)
/// * `base_texture_bytes` - Base skin texture as PNG bytes
/// * `skin_config_json` - Skin configuration JSON with cosmetic selections
/// * `asset_paths` - Paths for cosmetic assets (models, textures)
/// * `asset_bytes` - Asset file contents matching asset_paths
/// * `view_type` - Camera preset: "headshot", "isometric_head", "player_bust", "full_body_front", "front_right", "back_right"
/// * `width` - Output image width
/// * `height` - Output image height
///
/// # Returns
/// PNG image bytes on success, or an error string
#[wasm_bindgen]
pub fn render_hytale(
	model_json: &str,
	animation_json: &str,
	base_texture_bytes: &[u8],
	skin_config_json: &str,
	asset_paths: Vec<String>,
	asset_bytes: Vec<js_sys::Uint8Array>,
	view_type: &str,
	width: u32,
	height: u32,
) -> Result<Vec<u8>, JsValue> {
	let asset_bytes_vec: Vec<Vec<u8>> = asset_bytes.into_iter().map(|b| b.to_vec()).collect();
	let provider = MemoryAssetProvider::new(asset_paths, asset_bytes_vec)
		.map_err(|e| JsValue::from_str(&format!("Asset map error: {}", e)))?;

	let registry = std::sync::Arc::new(
		crate::cosmetics::CosmeticRegistry::load_from_provider(&provider, "assets/Common")
			.map_err(|e| JsValue::from_str(&format!("Registry load error: {}", e)))?,
	);

	let fallbacks = provider
		.load_bytes("assets/Common/Cosmetics/CharacterCreator/HaircutFallbacks.json")
		.ok()
		.and_then(|bytes| {
			serde_json::from_slice::<std::collections::HashMap<String, String>>(&bytes).ok()
		})
		.unwrap_or_else(std::collections::HashMap::new);

	let model = models::parse_blockymodel(model_json)
		.map_err(|e| JsValue::from_str(&format!("Model parse error: {}", e)))?;
	let animation = models::parse_blockyanim(animation_json)
		.map_err(|e| JsValue::from_str(&format!("Animation parse error: {}", e)))?;

	let mut renderer = render_pipeline::BodyRenderer::new_from_data(
		model,
		animation,
		registry,
		fallbacks,
		(256, 256),
	)
	.map_err(|e| JsValue::from_str(&format!("Renderer init error: {}", e)))?;

	let skin_config = skin::SkinConfig::from_str(skin_config_json)
		.map_err(|e| JsValue::from_str(&format!("Skin config error: {}", e)))?;

	let mut provider = provider;

	renderer
		.apply_skin_with_provider(&skin_config, &mut provider, "assets/Common")
		.map_err(|e| JsValue::from_str(&format!("Skin config error: {}", e)))?;

	let base_texture = texture::Texture::from_bytes(base_texture_bytes)
		.map_err(|e| JsValue::from_str(&format!("Base texture load error: {}", e)))?;

	let cam: Box<dyn camera::CameraProjection> = match view_type {
		"headshot" => Box::new(camera::PerspectiveCamera::headshot()),
		"isometric_head" => Box::new(camera::PerspectiveCamera::isometric_head()),
		"player_bust" => Box::new(camera::PerspectiveCamera::player_bust()),
		"full_body_front" => Box::new(camera::Camera::full_body_front()),
		"front_right" => Box::new(camera::Camera::front_right_view()),
		"back_right" => Box::new(camera::Camera::back_right_view()),
		_ => Box::new(camera::PerspectiveCamera::headshot()),
	};

	let image = renderer::render_scene_tinted(
		&renderer.faces,
		&base_texture,
		cam.as_ref(),
		width,
		height,
		&renderer.tint_config,
	)
	.map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

	output::export_png_bytes(&image).map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
}

/// Get available view types as a JSON array
#[wasm_bindgen]
pub fn get_available_view_types() -> String {
	r#"["headshot", "isometric_head", "player_bust", "full_body_front", "front_right", "back_right"]"#
        .to_string()
}
