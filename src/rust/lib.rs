extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod hytale;
mod minecraft;
mod text_avatar;
mod utils;

use hytale::HytaleSkin;
use image::DynamicImage;
use js_sys::Uint8Array;
use minecraft::*;
use std::io::Cursor;
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
	fn render_minecraft(
		self,
		img: &MinecraftSkin,
		size: u32,
		options: RenderOptions,
	) -> DynamicImage {
		match self {
			RenderType::Avatar => img
				.get_part(Layer::Bottom, BodyPart::Head, options.model)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Helm => img
				.get_part(Layer::Both, BodyPart::Head, options.model)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Cube => img.render_cube(size, options),
			RenderType::Body => img.render_body(options).resize(
				size,
				size * 2,
				image::imageops::FilterType::Nearest,
			),
			RenderType::Bust => img.render_body(options).crop(0, 0, 16, 16).resize(
				size,
				size,
				image::imageops::FilterType::Nearest,
			),
			RenderType::Cape => {
				img.get_cape()
					.resize(size, size, image::imageops::FilterType::Nearest)
			}
		}
	}

	fn render_hytale(self, img: &HytaleSkin, size: u32, options: RenderOptions) -> DynamicImage {
		use hytale::{BodyPart as HytaleBodyPart, Layer as HytaleLayer};

		match self {
			RenderType::Avatar => img
				.get_part(
					HytaleLayer::Bottom,
					HytaleBodyPart::Head,
					options.model.into(),
				)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Helm => img
				.get_part(
					HytaleLayer::Both,
					HytaleBodyPart::Head,
					options.model.into(),
				)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Cube => img.render_cube(size, options.into()),
			RenderType::Body => img.render_body(options.into()).resize(
				size,
				size * 2,
				image::imageops::FilterType::Nearest,
			),
			RenderType::Bust => img.render_body(options.into()).crop(0, 0, 16, 16).resize(
				size,
				size,
				image::imageops::FilterType::Nearest,
			),
			RenderType::Cape => {
				img.get_cape()
					.resize(size, size, image::imageops::FilterType::Nearest)
			}
		}
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

	let image_copy = skin_image.to_vec();

	let skin_result = image::load_from_memory_with_format(&image_copy, image::ImageFormat::Png);
	match skin_result {
		Ok(skin_img) => {
			let options = RenderOptions {
				armored,
				model: if slim {
					SkinModel::Slim
				} else {
					SkinModel::Regular
				},
			};

			let rendered = match game_type {
				Game::Minecraft => {
					let skin = MinecraftSkin::new(skin_img);
					render_type.render_minecraft(&skin, size, options)
				}
				Game::Hytale => {
					let skin = HytaleSkin::new(skin_img);
					render_type.render_hytale(&skin, size, options)
				}
			};

			// Better heuristic: ~4 bytes per pixel uncompressed, 50% compression ratio
			// For body renders, the height is 2x the size
			let estimated_size = (size * size * 2).max(4096) as usize;
			let mut result = Cursor::new(Vec::with_capacity(estimated_size));
			match rendered.write_to(&mut result, image::ImageFormat::Png) {
				Ok(()) => Ok(Uint8Array::from(&result.get_ref()[..])),
				Err(_err) => Err(js_sys::Error::new("Couldn't save resized skin.").into()),
			}
		}
		Err(_err) => Err(js_sys::Error::new("Couldn't load skin.").into()),
	}
}

/// Render a text-based avatar with username initials and deterministic background color
/// TEMPORARY: Used for Hytale until real skin support is implemented
#[wasm_bindgen]
pub fn render_text_avatar(username: String, size: u32) -> Result<Uint8Array, JsValue> {
	let image = text_avatar::render_text_avatar(&username, size);
	let dynamic = DynamicImage::ImageRgba8(image);

	let estimated_size = (size * size * 2).max(4096) as usize;
	let mut result = Cursor::new(Vec::with_capacity(estimated_size));
	match dynamic.write_to(&mut result, image::ImageFormat::Png) {
		Ok(()) => Ok(Uint8Array::from(&result.get_ref()[..])),
		Err(_err) => Err(js_sys::Error::new("Couldn't render text avatar.").into()),
	}
}

/// Render a Hytale character using the full 3D renderer
///
/// This function wraps HytaleSkinRenderer's WASM API for use in Crafthead.
/// It accepts pre-loaded model JSON, animation JSON, and texture bytes.
///
/// # Arguments
/// * `model_json` - BlockyModel JSON string (e.g., Player.blockymodel contents)
/// * `animation_json` - BlockyAnimation JSON string (e.g., Idle.blockyanim contents)
/// * `texture_bytes` - PNG texture data as Uint8Array
/// * `view_type` - View type: "avatar", "helm", "cube", "body", "bust"
/// * `size` - Output image size (width for square, width for body)
///
/// # Returns
/// PNG image bytes as Uint8Array on success
#[wasm_bindgen]
pub fn render_hytale_3d(
	model_json: String,
	animation_json: String,
	texture_bytes: Uint8Array,
	cosmetics: JsValue,
	base_skin_tint: Option<Vec<String>>,
	base_skin_tint_texture: Option<Uint8Array>,
	view_type: String,
	size: u32,
) -> Result<Uint8Array, JsValue> {
	let texture_vec = texture_bytes.to_vec();

	// Determine output dimensions based on view type
	let (width, height) = match view_type.as_str() {
		"body" | "full_body_front" => (size, size * 2),
		_ => (size, size),
	};

	// Map Crafthead view types to HytaleSkinRenderer camera types
	let camera_type = match view_type.as_str() {
		"avatar" | "helm" | "headshot" => "headshot",
		"cube" | "isometric_head" => "isometric_head",
		"bust" | "player_bust" => "player_bust",
		"body" | "full_body_front" => "full_body_front",
		_ => "headshot",
	};

	// Call HytaleSkinRenderer's WASM function
	let png_bytes = hytale_skin_renderer::wasm::render_hytale_with_cosmetics(
		&model_json,
		&animation_json,
		&texture_vec,
		cosmetics,
		base_skin_tint,
		base_skin_tint_texture.map(|b| b.to_vec()),
		camera_type,
		width,
		height,
	)?;

	Ok(Uint8Array::from(&png_bytes[..]))
}

#[cfg(test)]
mod tests {
	use super::*;

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
