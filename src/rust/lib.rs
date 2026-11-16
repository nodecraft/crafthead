extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod cube;
mod hytale;
mod minecraft;
mod text_avatar;
mod utils;

use hytale::HytaleSkin;
use hytale_skin_renderer::asset_provider::MemoryAssetProvider;
use hytale_skin_renderer::output::export_png_bytes;
use hytale_skin_renderer::render_pipeline::RenderContext;
use image::RgbaImage;
use js_sys::Uint8Array;
use minecraft::*;
use std::cell::RefCell;
use utils::scale_nearest;
use wasm_bindgen::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
enum RenderType {
	Avatar,
	Helm,
	Cube,
	Body,
	Bust,
	Cape,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Game {
	Minecraft,
	Hytale,
}

pub(crate) struct RenderOptions {
	pub armored: bool,
	pub model: SkinModel,
}

impl RenderType {
	fn render_minecraft(self, img: &MinecraftSkin, size: u32, options: RenderOptions) -> RgbaImage {
		let (unscaled, max_height) = match self {
			RenderType::Cube => return img.render_cube(size, options),
			RenderType::Avatar => (
				img.get_part(Layer::Bottom, BodyPart::Head, options.model),
				size,
			),
			RenderType::Helm => (
				img.get_part(Layer::Both, BodyPart::Head, options.model),
				size,
			),
			RenderType::Body => (img.render_body(options), size * 2),
			RenderType::Bust => (img.render_body(options).crop(0, 0, 16, 16), size),
			RenderType::Cape => (img.get_cape(), size),
		};
		scale_nearest(&unscaled.into_rgba8(), size, max_height)
	}

	fn render_hytale(self, img: &HytaleSkin, size: u32, options: RenderOptions) -> RgbaImage {
		use hytale::{BodyPart as HytaleBodyPart, Layer as HytaleLayer};

		let (unscaled, max_height) = match self {
			RenderType::Cube => return img.render_cube(size, options.into()),
			RenderType::Avatar => (
				img.get_part(
					HytaleLayer::Bottom,
					HytaleBodyPart::Head,
					options.model.into(),
				),
				size,
			),
			RenderType::Helm => (
				img.get_part(
					HytaleLayer::Both,
					HytaleBodyPart::Head,
					options.model.into(),
				),
				size,
			),
			RenderType::Body => (img.render_body(options.into()), size * 2),
			RenderType::Bust => (img.render_body(options.into()).crop(0, 0, 16, 16), size),
			RenderType::Cape => (img.get_cape(), size),
		};
		scale_nearest(&unscaled.into_rgba8(), size, max_height)
	}
}

// Conversion from skin::SkinModel to hytale::SkinModel
impl From<SkinModel> for hytale::SkinModel {
	fn from(model: SkinModel) -> Self {
		match model {
			SkinModel::Slim => hytale::SkinModel::Slim,
			SkinModel::Regular => hytale::SkinModel::Regular,
		}
	}
}

// Conversion from RenderOptions to hytale-compatible options
impl From<RenderOptions> for crate::hytale::RenderOptions {
	fn from(opts: RenderOptions) -> Self {
		crate::hytale::RenderOptions {
			armored: opts.armored,
			model: opts.model.into(),
		}
	}
}

#[inline]
fn what_to_render_type(what: String) -> Option<RenderType> {
	match what.as_str() {
		"avatar" => Some(RenderType::Avatar),
		"helm" => Some(RenderType::Helm),
		"cube" => Some(RenderType::Cube),
		"body" => Some(RenderType::Body),
		"bust" => Some(RenderType::Bust),
		"cape" => Some(RenderType::Cape),
		_ => None,
	}
}

#[inline]
fn string_to_game(game: &str) -> Option<Game> {
	match game {
		"minecraft" => Some(Game::Minecraft),
		"hytale" => Some(Game::Hytale),
		_ => None,
	}
}

#[wasm_bindgen]
pub fn get_rendered_image(
	skin_image: Uint8Array,
	size: u32,
	what: String,
	armored: bool,
	slim: bool,
	game: String,
) -> Result<Uint8Array, JsValue> {
	let game_type = string_to_game(&game);
	if game_type.is_none() {
		return Err(js_sys::Error::new(&format!("Unknown game '{}'.", game)).into());
	}
	let game_type = game_type.unwrap();

	let render_type = what_to_render_type(what);
	if render_type.is_none() {
		return Err(js_sys::Error::new("Invalid render type.").into());
	}
	let render_type = render_type.unwrap();

	let options = RenderOptions {
		armored,
		model: if slim {
			SkinModel::Slim
		} else {
			SkinModel::Regular
		},
	};

	match render_skin(&skin_image.to_vec(), size, render_type, options, game_type) {
		Ok(png) => Ok(Uint8Array::from(&png[..])),
		Err(message) => Err(js_sys::Error::new(message).into()),
	}
}

/// Decode a skin PNG and render the requested view of it as PNG bytes.
fn render_skin(
	skin_png: &[u8],
	size: u32,
	render_type: RenderType,
	options: RenderOptions,
	game: Game,
) -> Result<Vec<u8>, &'static str> {
	let skin_img = image::load_from_memory_with_format(skin_png, image::ImageFormat::Png)
		.map_err(|_| "Couldn't load skin.")?;

	let rendered = match game {
		Game::Minecraft => {
			let skin = MinecraftSkin::new(skin_img);
			render_type.render_minecraft(&skin, size, options)
		}
		Game::Hytale => {
			let skin = HytaleSkin::new(skin_img);
			render_type.render_hytale(&skin, size, options)
		}
	};

	export_png_bytes(&rendered).map_err(|_| "Couldn't save resized skin.")
}

/// Render a text-based avatar with username initials and deterministic background color
/// TEMPORARY: Used for Hytale until real skin support is implemented
#[wasm_bindgen]
pub fn render_text_avatar(username: String, size: u32) -> Result<Uint8Array, JsValue> {
	let image = text_avatar::render_text_avatar(&username, size);

	match export_png_bytes(&image) {
		Ok(png) => Ok(Uint8Array::from(&png[..])),
		Err(_err) => Err(js_sys::Error::new("Couldn't render text avatar.").into()),
	}
}

const HYTALE_ASSET_ROOT: &str = "assets/Common";

thread_local! {
	// Lives as long as the WASM instance, so each worker isolate parses the shared assets once.
	static HYTALE_CONTEXT: RefCell<Option<RenderContext>> = const { RefCell::new(None) };
}

fn hytale_error(error: hytale_skin_renderer::Error) -> JsValue {
	js_sys::Error::new(&error.to_string()).into()
}

fn memory_assets(
	asset_paths: Vec<String>,
	asset_bytes: Vec<Uint8Array>,
) -> Result<MemoryAssetProvider, JsValue> {
	let asset_bytes = asset_bytes.iter().map(Uint8Array::to_vec).collect();
	MemoryAssetProvider::new(asset_paths, asset_bytes).map_err(hytale_error)
}

/// Load the assets every Hytale render shares. Call once before `render_hytale_3d`.
///
/// # Arguments
/// * `model_json` - BlockyModel JSON string (Player.blockymodel contents)
/// * `animation_json` - BlockyAnimation JSON string (Idle.blockyanim contents)
/// * `asset_paths` - Paths of the bundled CharacterCreator/particle JSON files
/// * `asset_bytes` - File contents matching `asset_paths`
#[wasm_bindgen]
pub fn load_hytale_context(
	model_json: String,
	animation_json: String,
	asset_paths: Vec<String>,
	asset_bytes: Vec<Uint8Array>,
) -> Result<(), JsValue> {
	let shared_assets = memory_assets(asset_paths, asset_bytes)?;
	let context = RenderContext::new(
		&model_json,
		&animation_json,
		Box::new(shared_assets),
		HYTALE_ASSET_ROOT,
	)
	.map_err(hytale_error)?;
	HYTALE_CONTEXT.with(|slot| *slot.borrow_mut() = Some(context));
	Ok(())
}

/// Render a Hytale character using the full 3D renderer
///
/// # Arguments
/// * `texture_bytes` - Base skin texture as PNG bytes
/// * `skin_config_json` - Skin configuration JSON with cosmetic selections
/// * `asset_paths` - Paths of the models, textures and gradients this skin uses
/// * `asset_bytes` - File contents matching `asset_paths`
/// * `view_type` - View type: "avatar", "helm", "cube", "body", "bust"
/// * `size` - Output image width; body renders are twice as tall
///
/// # Returns
/// PNG image bytes as Uint8Array on success
#[wasm_bindgen]
pub fn render_hytale_3d(
	texture_bytes: Uint8Array,
	skin_config_json: String,
	asset_paths: Vec<String>,
	asset_bytes: Vec<Uint8Array>,
	view_type: String,
	size: u32,
) -> Result<Uint8Array, JsValue> {
	let (width, height) = match view_type.as_str() {
		"body" | "full_body_front" => (size, size * 2),
		_ => (size, size),
	};

	let camera_type = match view_type.as_str() {
		"avatar" | "helm" | "headshot" => "headshot",
		"cube" | "isometric_head" => "isometric_head",
		"bust" | "player_bust" => "player_bust",
		"body" | "full_body_front" => "full_body_front",
		_ => "headshot",
	};

	let assets = memory_assets(asset_paths, asset_bytes)?;
	let png_bytes = HYTALE_CONTEXT.with(|slot| {
		let slot = slot.borrow();
		let Some(context) = slot.as_ref() else {
			return Err(JsValue::from(js_sys::Error::new(
				"Hytale context not loaded; call load_hytale_context first.",
			)));
		};
		context
			.render_png(
				&skin_config_json,
				&texture_bytes.to_vec(),
				&assets,
				camera_type,
				width,
				height,
			)
			.map_err(hytale_error)
	})?;

	Ok(Uint8Array::from(&png_bytes[..]))
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::Rgba;

	/// A modern skin where every pixel encodes its own position, as PNG bytes.
	fn positional_skin() -> (RgbaImage, Vec<u8>) {
		let skin = RgbaImage::from_fn(64, 64, |x, y| Rgba([x as u8 * 4, y as u8 * 4, 77, 255]));
		let png = export_png_bytes(&skin).unwrap();
		(skin, png)
	}

	fn render(png: &[u8], size: u32, render_type: RenderType, slim: bool) -> RgbaImage {
		let options = RenderOptions {
			armored: false,
			model: if slim {
				SkinModel::Slim
			} else {
				SkinModel::Regular
			},
		};
		let rendered = render_skin(png, size, render_type, options, Game::Minecraft).unwrap();
		image::load_from_memory(&rendered).unwrap().into_rgba8()
	}

	#[test]
	fn test_render_skin_avatar_is_the_scaled_face() {
		let (skin, png) = positional_skin();
		let avatar = render(&png, 80, RenderType::Avatar, false);

		assert_eq!(avatar.dimensions(), (80, 80));
		// The face is the 8x8 block at (8, 8); at 10x scale each texel covers 10 pixels.
		for (x, y) in [(0, 0), (9, 9), (10, 0), (45, 72), (79, 79)] {
			assert_eq!(
				avatar.get_pixel(x, y),
				skin.get_pixel(8 + x / 10, 8 + y / 10),
				"pixel ({}, {})",
				x,
				y
			);
		}
	}

	#[test]
	fn test_render_skin_output_sizes() {
		let (_, png) = positional_skin();

		assert_eq!(
			render(&png, 40, RenderType::Body, false).dimensions(),
			(40, 80)
		);
		// Slim bodies are 14 wide instead of 16, and scaling keeps that aspect ratio.
		assert_eq!(
			render(&png, 40, RenderType::Body, true).dimensions(),
			(35, 80)
		);
		assert_eq!(
			render(&png, 40, RenderType::Bust, false).dimensions(),
			(40, 40)
		);
		assert_eq!(
			render(&png, 40, RenderType::Cube, false).dimensions(),
			(40, 40)
		);
		assert_eq!(
			render(&png, 32, RenderType::Cape, false).dimensions(),
			(20, 32)
		);
	}

	#[test]
	fn test_render_skin_cube_leaves_the_corners_empty() {
		let (_, png) = positional_skin();
		let cube = render(&png, 100, RenderType::Cube, false);

		assert_eq!(cube.get_pixel(0, 0)[3], 0);
		assert_eq!(cube.get_pixel(99, 99)[3], 0);
		assert_eq!(cube.get_pixel(50, 50)[3], 255);
	}

	#[test]
	fn test_render_skin_rejects_data_that_is_not_a_png() {
		let options = RenderOptions {
			armored: false,
			model: SkinModel::Regular,
		};
		let result = render_skin(
			b"not a png",
			64,
			RenderType::Avatar,
			options,
			Game::Minecraft,
		);
		assert_eq!(result, Err("Couldn't load skin."));
	}

	#[test]
	fn test_what_to_render_type_avatar() {
		assert_eq!(
			what_to_render_type("avatar".to_string()),
			Some(RenderType::Avatar)
		);
	}

	#[test]
	fn test_what_to_render_type_helm() {
		assert_eq!(
			what_to_render_type("helm".to_string()),
			Some(RenderType::Helm)
		);
	}

	#[test]
	fn test_what_to_render_type_cube() {
		assert_eq!(
			what_to_render_type("cube".to_string()),
			Some(RenderType::Cube)
		);
	}

	#[test]
	fn test_what_to_render_type_body() {
		assert_eq!(
			what_to_render_type("body".to_string()),
			Some(RenderType::Body)
		);
	}

	#[test]
	fn test_what_to_render_type_bust() {
		assert_eq!(
			what_to_render_type("bust".to_string()),
			Some(RenderType::Bust)
		);
	}

	#[test]
	fn test_what_to_render_type_cape() {
		assert_eq!(
			what_to_render_type("cape".to_string()),
			Some(RenderType::Cape)
		);
	}

	#[test]
	fn test_what_to_render_type_invalid() {
		assert_eq!(what_to_render_type("invalid".to_string()), None);
	}

	#[test]
	fn test_string_to_game_minecraft() {
		assert_eq!(string_to_game("minecraft"), Some(Game::Minecraft));
	}

	#[test]
	fn test_string_to_game_hytale() {
		assert_eq!(string_to_game("hytale"), Some(Game::Hytale));
	}

	#[test]
	fn test_string_to_game_invalid() {
		assert_eq!(string_to_game("unknown"), None);
	}
}
