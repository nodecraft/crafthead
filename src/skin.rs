extern crate image;

use image::{DynamicImage, GenericImageView, imageops, GenericImage, Rgba};

pub(crate) struct MinecraftSkin(DynamicImage);

pub(crate) enum MinecraftSkinVersion {
    Classic, // 64x32
    Modern, // 64x64
    Invalid
}

pub(crate) enum Layer {
    Bottom,
    Top,
    Both,
}

pub(crate) enum BodyPart {
    Head,
    Body,
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
}

impl MinecraftSkin {
    pub fn new(skin: DynamicImage) -> MinecraftSkin {
        MinecraftSkin(skin)
    }

    fn version(&self) -> MinecraftSkinVersion {
        match self.0.dimensions() {
            (64, 32) => MinecraftSkinVersion::Classic,
            (64, 64) => MinecraftSkinVersion::Modern,
            _        => MinecraftSkinVersion::Invalid
        }
    }

    pub(crate) fn get_part(&self, layer: &Layer, part: &BodyPart) -> DynamicImage {
        // TODO: This code is from #6 but doesn't work for old skins and has transparency problems
        //       Thankfully we can easily tell if it's an old skin or not...
        match layer {
            Layer::Both => {
                let mut bottom = self.get_part(&Layer::Bottom, part);
                let mut top = self.get_part(&Layer::Top, part);
                MinecraftSkin::fast_overlay(&mut bottom, &top, 0, 0);
                bottom
            },
            Layer::Bottom => {
                match part {
                    BodyPart::Head => self.0.crop_imm(8, 8, 8, 8),
                    BodyPart::Body => self.0.crop_imm(20, 20, 8, 12),
                    BodyPart::ArmLeft => self.0.crop_imm(36, 52, 4, 12),
                    BodyPart::ArmRight => self.0.crop_imm(44, 20, 4, 12),
                    BodyPart::LegLeft => self.0.crop_imm(20, 52, 4, 12),
                    BodyPart::LegRight => self.0.crop_imm(4, 20, 4, 12),
                }
            },
            Layer::Top => {
                let mut portion = match part {
                    BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
                    BodyPart::Body => self.0.crop_imm(20, 36, 8, 12),
                    BodyPart::ArmLeft => self.0.crop_imm(52, 52, 4, 12),
                    BodyPart::ArmRight => self.0.crop_imm(44, 36, 4, 12),
                    BodyPart::LegLeft => self.0.crop_imm(4, 52, 4, 12),
                    BodyPart::LegRight => self.0.crop_imm(4, 36, 4, 12),
                };
                MinecraftSkin::apply_minecraft_transparency(&mut portion);
                portion
            },
        }
    }

    fn fast_overlay(bottom: &mut DynamicImage, top: &DynamicImage, x: u32, y: u32) {
        // All but a straight port of https://github.com/minotar/imgd/blob/master/process.go#L386
        // to Rust.
        let bottom_dims = bottom.dimensions();
        let top_dims = top.dimensions();

        // Crop our top image if we're going out of bounds
        let (range_width, range_height) = imageops::overlay_bounds(bottom_dims, top_dims, x, y);

        for top_y in 0..range_height {
            for top_x in 0..range_width {
                let mut p = top.get_pixel(top_x, top_y);
                if p[3] != 0 {
                    p[3] = 0xFF;
                    bottom.put_pixel(x + top_x, y + top_y, p);
                }
            }
        }
    }

    fn is_region_transparent(img: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> bool {
        for cy in y..y+height {
            for cx in x..x+width {
                let mut p = img.get_pixel(cx, cy);
                if p[3] < 128 {
                    return true
                }
            }
        }
        return false
    }

    fn apply_minecraft_transparency(img: &mut DynamicImage) {
        let (width, height) = img.dimensions();
        MinecraftSkin::apply_minecraft_transparency_region(img, 0, 0, width, height);
    }

    fn apply_minecraft_transparency_region(img: &mut DynamicImage, x: u32, y: u32, width: u32, height: u32) {
        if MinecraftSkin::is_region_transparent(img, x, y, width, height) {
            return
        }

        for cy in y..y+height {
            for cx in x..x+width {
                let mut p= img.get_pixel(cx, cy);
                p[3] = 0x00;
                img.put_pixel(cx, cy, p);
            }
        }
    }
}