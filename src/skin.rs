extern crate image;

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage, imageops};
use imageproc::geometric_transformations::{Projection, Interpolation, warp_into};
use crate::skin::Layer::Bottom;
use crate::skin::BodyPart::{ArmLeft, LegLeft, Body, Head};
use crate::utils::{apply_minecraft_transparency, fast_overlay};

pub(crate) struct MinecraftSkin(DynamicImage);

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum MinecraftSkinVersion {
    Classic, // 64x32
    Modern, // 64x64
    Invalid
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Layer {
    Bottom,
    Top,
    Both,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum BodyPart {
    Head,
    Body,
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
}

const skew_a: f32 = 26.0 / 45.0;  // 0.57777777
const skew_b: f32 = skew_a * 2.0; // 1.15555555

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

    pub(crate) fn get_part(&self, layer: Layer, part: BodyPart) -> DynamicImage {
        let arm_width = match self.version() {
            MinecraftSkinVersion::Classic => 3,
            _                             => 4
        };

        match layer {
            Layer::Both => {
                if self.version() != MinecraftSkinVersion::Modern && part != Head {
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
                    BodyPart::ArmRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(36, 52, arm_width, 12),
                            _                            => self.get_part(Bottom, ArmLeft).fliph()
                        }
                    },
                    BodyPart::ArmLeft => self.0.crop_imm(44, 20, 4, 12),
                    BodyPart::LegRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(20, 52, 4, 12),
                            _                            => self.get_part(Bottom, LegLeft).fliph()
                        }
                    },
                    BodyPart::LegLeft => self.0.crop_imm(4, 20, 4, 12),
                }
            },
            Layer::Top => {
                match part {
                    BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
                    BodyPart::Body => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(20, 36, 8, 12),
                            _                            => self.get_part(Bottom, Body)
                        }
                    },
                    BodyPart::ArmLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(52, 52, arm_width, 12),
                            _                            => self.get_part(Bottom, ArmLeft)
                        }
                    },
                    BodyPart::ArmRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(44, 36, arm_width, 12),
                            _                            => self.get_part(Bottom, ArmLeft).fliph(),
                        }
                    },
                    BodyPart::LegLeft => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(4, 52, 4, 12),
                            _                            => self.get_part(Bottom, LegLeft),
                        }
                    },
                    BodyPart::LegRight => {
                        match self.version() {
                            MinecraftSkinVersion::Modern => self.0.crop_imm(4, 36, 4, 12),
                            _                            => self.get_part(Bottom, LegLeft).fliph(),
                        }
                    },
                }
            },
        }
    }

    pub(crate) fn render_body(&self, overlay: bool) -> DynamicImage {
        let layer_type = match overlay {
            true  => Layer::Both,
            false => Layer::Bottom
        };

        let mut image = RgbaImage::new(16, 32);

        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::Head), 4, 0);
        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::Body), 4, 8);
        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::ArmLeft), 0, 8);
        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::ArmRight), 12, 8);
        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::LegLeft), 4, 20);
        imageops::overlay(&mut image, &self.get_part(layer_type, BodyPart::LegRight), 8, 20);

        DynamicImage::ImageRgba8(image)
    }

    pub(crate) fn render_cube(&self, overlay: bool, width: u32) -> DynamicImage {
        let scale = (width as f32) / 20.0 as f32;
        let height = (18.5 * scale).ceil() as u32;
        let layer_type = match overlay {
            true  => Layer::Both,
            false => Layer::Bottom
        };
        let mut render = RgbaImage::new(width, height);

        let z_offset = scale * 3.0;
        let x_offset = scale * 2.0;

        let head_orig_top   = self.0.crop_imm(8, 0, 8, 8);
        let head_orig_right = self.0.crop_imm(0, 8, 8, 8);
        let head_orig_front = self.0.crop_imm(8, 8, 8, 8);

        // The warp_into function clears every part of the output image that is not part of the pre-image.
        // As a workaround, we ask warp_into to draw into a scratch image, overlay the final image with the
        // scratch image, and let the scratch be overwritten.
        let mut scratch = RgbaImage::new(width, height);

        // head top
        let head_top_skew = Projection::from_matrix([
            1.0,     1.0,    0.0,
            -skew_a, skew_a, 0.0,
            0.0,     0.0,    1.0,
        ]).unwrap() * Projection::translate(-0.5 - z_offset, x_offset + z_offset - 0.5) * Projection::scale(scale, scale + (1.0 / 8.0));
        warp_into(&head_orig_top.into_rgba8(), &head_top_skew, Interpolation::Nearest, Rgba([0, 0, 0, 0]), &mut scratch);
        imageops::overlay(&mut render, &scratch, 0, 0);

        // head front
        let head_front_skew = Projection::from_matrix([
            1.0,     0.0,    0.0,
            -skew_a, skew_b, skew_a,
            0.0,     0.0,    1.0,
        ]).unwrap() * Projection::translate(x_offset + 7.5 * scale - 0.5, (x_offset + 8.0 * scale) + z_offset - 0.5) * Projection::scale(scale, scale);
        warp_into(&head_orig_front.into_rgba8(), &head_front_skew, Interpolation::Nearest, Rgba([0, 0, 0, 0]), &mut scratch);
        imageops::overlay(&mut render, &scratch, 0, 0);

        // head right
        let head_right_skew = Projection::from_matrix([
            1.0,     0.0,    0.0,
            skew_a,  skew_b, 0.0,
            0.0,     0.0,    1.0,
        ]).unwrap() * Projection::translate(x_offset - (scale / 2.0), z_offset + scale) * Projection::scale(scale + (0.5 / 8.0), scale + (1.0 / 8.0));
        warp_into(&head_orig_right.into_rgba8(), &head_right_skew, Interpolation::Nearest, Rgba([0, 0, 0, 0]), &mut scratch);
        imageops::overlay(&mut render, &scratch, 0, 0);

        DynamicImage::ImageRgba8(render)
    }
}
