extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod skin;
mod utils;

use image::DynamicImage;
use js_sys::Uint8Array;
use skin::*;
use wasm_bindgen::prelude::*;

enum RenderType {
	Avatar,
	Helm,
	Cube,
	Body,
	Bust,
	Cape,
}

struct RenderOptions {
	armored: bool,
	model: SkinModel,
}

impl RenderType {
	fn render(self, img: &MinecraftSkin, size: u32, options: RenderOptions) -> DynamicImage {
		match self {
			RenderType::Avatar => img
				.get_part(Layer::Bottom, BodyPart::Head, options.model)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Helm => img
				.get_part(Layer::Both, BodyPart::Head, options.model)
				.resize(size, size, image::imageops::FilterType::Nearest),
			RenderType::Cube => img.render_cube(true, size),
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
}

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

#[wasm_bindgen]
pub fn get_rendered_image(
	skin_image: Uint8Array,
	size: u32,
	what: String,
	armored: bool,
	slim: bool,
) -> Result<Uint8Array, JsValue> {
	let render_type = what_to_render_type(what);
	if render_type.is_none() {
		return Err(js_sys::Error::new("Invalid render type.").into());
	}

	let image_copy = skin_image.to_vec();

	let skin_result = image::load_from_memory_with_format(&image_copy, image::ImageFormat::Png);
	match skin_result {
		Ok(skin_img) => {
			let skin = MinecraftSkin::new(skin_img);
			let options = match slim {
				true => RenderOptions {
					armored,
					model: SkinModel::Slim,
				},
				false => RenderOptions {
					armored,
					model: SkinModel::Regular,
				},
			};
			let rendered = render_type.unwrap().render(&skin, size, options);
			let mut result = std::io::Cursor::new(Vec::with_capacity(1024));
			return match rendered.write_to(&mut result, image::ImageFormat::Png) {
				Ok(()) => Ok(Uint8Array::from(&result.get_ref()[..])),
				Err(_err) => Err(js_sys::Error::new("Couldn't save resized skin.").into()),
			};
		}
		Err(_err) => {
			return Err(js_sys::Error::new("Couldn't load skin.").into());
		}
	}
}
