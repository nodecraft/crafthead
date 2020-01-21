extern crate cfg_if;
extern crate image;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use js_sys::Uint8Array;
use image::{DynamicImage, imageops};
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

struct Skin(DynamicImage);

enum Layer {
    Bottom,
    Top,
    Both,
}

enum BodyPart {
    Head,
    Body,
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
}

impl Skin {
    fn get_part(&mut self, layer: &Layer, part: &BodyPart) -> DynamicImage {
        match layer {
            Layer::Both => {
                let mut bottom = self.get_part(&Layer::Bottom, part);
                let top = self.get_part(&Layer::Top, part);
                imageops::overlay(&mut bottom, &top, 0, 0);
                bottom
            },
            Layer::Bottom => {
                match part {
                    BodyPart::Head => self.0.crop(8, 8, 8, 8),
                    BodyPart::Body => self.0.crop(20, 20, 8, 12),
                    BodyPart::ArmLeft => self.0.crop(36, 52, 4, 12),
                    BodyPart::ArmRight => self.0.crop(44, 20, 4, 12),
                    BodyPart::LegLeft => self.0.crop(20, 52, 4, 12),
                    BodyPart::LegRight => self.0.crop(4, 20, 4, 12),
                }
            },
            Layer::Top => {
                match part {
                    BodyPart::Head => self.0.crop(40, 8, 8, 8),
                    BodyPart::Body => self.0.crop(20, 36, 8, 12),
                    BodyPart::ArmLeft => self.0.crop(52, 52, 4, 12),
                    BodyPart::ArmRight => self.0.crop(44, 36, 4, 12),
                    BodyPart::LegLeft => self.0.crop(4, 52, 4, 12),
                    BodyPart::LegRight => self.0.crop(4, 36, 4, 12),
                }
            },
        }
    }
}

#[wasm_bindgen]
pub fn get_minecraft_head(skin_image: Uint8Array, size: u32) -> Uint8Array {
    let image_copy = skin_image.to_vec();

    let decoded_skin =
        match image::load_from_memory_with_format(&image_copy, image::ImageFormat::PNG) {
            Ok(skin) => skin,
            Err(e) => {
                let msg = format!("error whilst loading image: {}", e);
                wasm_bindgen::throw_str(&msg);
            }
        };
    let just_head = Skin(decoded_skin).get_part(&Layer::Both, &BodyPart::Head);
    let head = just_head.resize(size, size, imageops::FilterType::Nearest);

    // For "common" image sizes, 1KiB should be plenty (typically < 140px)
    let mut result = Vec::with_capacity(1024);
    head.write_to(&mut result, image::ImageFormat::PNG).unwrap();
    return Uint8Array::from(&result[..]);
}
