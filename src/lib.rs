extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn get_minecraft_head(skin_image: Uint8Array, size: u32) -> Result<Uint8Array, JsValue> {
    let image_copy = skin_image.to_vec();

    let skin_result = image::load_from_memory_with_format(&image_copy, image::ImageFormat::Png);
    match skin_result {
        Ok(skin) => {
            let head = crop_head(skin, size);
            let mut result = Vec::with_capacity(1024);
            return match head.write_to(&mut result, image::ImageFormat::Png) {
                Ok(()) => Ok(Uint8Array::from(&result[..])),
                Err(_err) => Err(js_sys::Error::new("Couldn't save resized skin.").into())
            };
        },
        Err(_err) => {
            return Err(js_sys::Error::new("Couldn't load skin.").into());
        }
    }
}

fn crop_head(skin: image::DynamicImage, size: u32) -> image::DynamicImage {
    let just_head = skin.crop_imm(8, 8, 8, 8);
    let head = just_head.resize(size, size, image::imageops::FilterType::Nearest);
    head
}