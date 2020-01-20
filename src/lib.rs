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
pub fn get_minecraft_head(skin_image: Uint8Array, size: u32) -> Uint8Array {
    let image_copy = skin_image.to_vec();

    let mut decoded_skin =
        match image::load_from_memory_with_format(&image_copy, image::ImageFormat::PNG) {
            Ok(skin) => skin,
            Err(e) => {
                let msg = format!("error whilst loading image: {}", e);
                wasm_bindgen::throw_str(&msg);
            }
        };
    let just_head = decoded_skin.crop(8, 8, 8, 8);
    let head = just_head.resize(size, size, image::imageops::FilterType::Nearest);

    // For "common" image sizes, 1KiB should be plenty (typically < 140px)
    let mut result = Vec::with_capacity(1024);
    head.write_to(&mut result, image::ImageFormat::PNG).unwrap();
    return Uint8Array::from(&result[..]);
}
