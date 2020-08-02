extern crate image;

use image::{DynamicImage, GenericImageView};
use crate::skin::Layer::Bottom;
use crate::skin::BodyPart::{ArmRight, LegRight, Body, Head};
use crate::utils::{apply_minecraft_transparency, fast_overlay};

pub(crate) struct MinecraftSkin(DynamicImage);

#[derive(PartialEq)]
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

#[derive(PartialEq)]
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

    pub(crate) fn get_part(&self, layer: Layer, part: &BodyPart) -> DynamicImage {
        match layer {
            Layer::Both => {
                if self.version() != MinecraftSkinVersion::Modern && *part != Head {
                    return self.get_part(Layer::Bottom, part);
                }

                let mut bottom = self.get_part(Layer::Bottom, part);
                let mut top = self.get_part(Layer::Top, part);
                apply_minecraft_transparency(&mut top);
                fast_overlay(&mut bottom, &top, 0, 0);
                bottom
            },
            Layer::Bottom => {
                match part {
                    BodyPart::Head => self.0.crop_imm(8, 8, 8, 8),
                    BodyPart::Body => self.0.crop_imm(20, 20, 8, 12),
                    BodyPart::ArmLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(36, 52, 4, 12),
                            _                            => self.get_part(Bottom, &ArmRight)
                        }
                    },
                    BodyPart::ArmRight => self.0.crop_imm(44, 20, 4, 12),
                    BodyPart::LegLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(20, 52, 4, 12),
                            _                            => self.get_part(Bottom, &LegRight)
                        }
                    },
                    BodyPart::LegRight => self.0.crop_imm(4, 20, 4, 12),
                }
            },
            Layer::Top => {
                match part {
                    BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
                    BodyPart::Body => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(20, 36, 8, 12),
                            _                            => self.get_part(Bottom, &Body)
                        }
                    },
                    BodyPart::ArmLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(52, 52, 4, 12),
                            _                            => self.get_part(Bottom, &ArmRight)
                        }
                    },
                    BodyPart::ArmRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(44, 36, 4, 12),
                            _                            => self.get_part(Bottom, &ArmRight),
                        }
                    },
                    BodyPart::LegLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(4, 52, 4, 12),
                            _                            => self.get_part(Bottom, &LegRight),
                        }
                    },
                    BodyPart::LegRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(4, 36, 4, 12),
                            _                            => self.get_part(Bottom, &LegRight),
                        }
                    },
                }
            },
        }
    }
}