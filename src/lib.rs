extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod utils;
mod skin;

use cfg_if::cfg_if;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use skin::*;
use image::DynamicImage;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

enum RenderType {
    Avatar,
    Helm,
    Cube,
    Body,
}

struct RenderOptions {
    armored: bool,
    model: SkinModel,
}

impl RenderType {
    fn render(self, img: &MinecraftSkin, size: u32, options: RenderOptions) -> DynamicImage {
        match self {
            RenderType::Avatar => img.get_part(Layer::Bottom, BodyPart::Head, options.model)
                .resize(size, size, image::imageops::FilterType::Nearest),
            RenderType::Helm   => img.get_part(Layer::Both, BodyPart::Head, options.model)
                .resize(size, size, image::imageops::FilterType::Nearest),
            RenderType::Body   => img.render_body(options)
                .resize(size, size * 2, image::imageops::FilterType::Nearest),
            RenderType::Cube   => img.render_cube(true, size),
        }
    }
}

fn what_to_render_type(what: String) -> Option<RenderType> {
    match what.as_str() {
        "avatar" => Some(RenderType::Avatar),
        "helm"   => Some(RenderType::Helm),
        "cube"   => Some(RenderType::Cube),
        "body"   => Some(RenderType::Body),
        _        => None
    }
}

#[wasm_bindgen]
pub fn get_rendered_image(skin_image: Uint8Array, size: u32, what: String, armored: bool, slim: bool) -> Result<Uint8Array, JsValue> {
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
                true =>  RenderOptions { armored, model: SkinModel::Slim },
                false => RenderOptions { armored, model: SkinModel::Regular }
            };
            let rendered = render_type.unwrap().render(&skin, size, options);
            let mut result = Vec::with_capacity(1024);
            return match rendered.write_to(&mut result, image::ImageFormat::Png) {
                Ok(()) => Ok(Uint8Array::from(&result[..])),
                Err(_err) => Err(js_sys::Error::new("Couldn't save resized skin.").into())
            };
        },
        Err(_err) => {
            return Err(js_sys::Error::new("Couldn't load skin.").into());
        }
    }
}