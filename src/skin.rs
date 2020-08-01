extern crate image;

use image::{DynamicImage, GenericImageView, imageops};

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
                let top = self.get_part(&Layer::Top, part);
                imageops::overlay(&mut bottom, &top, 0, 0);
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
                match part {
                    BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
                    BodyPart::Body => self.0.crop_imm(20, 36, 8, 12),
                    BodyPart::ArmLeft => self.0.crop_imm(52, 52, 4, 12),
                    BodyPart::ArmRight => self.0.crop_imm(44, 36, 4, 12),
                    BodyPart::LegLeft => self.0.crop_imm(4, 52, 4, 12),
                    BodyPart::LegRight => self.0.crop_imm(4, 36, 4, 12),
                }
            },
        }
    }
}