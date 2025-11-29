extern crate image;

use crate::skin::BodyPart::{ArmLeft, Body, Head, LegLeft};
use crate::skin::Layer::Bottom;
use crate::utils::{apply_minecraft_transparency, fast_overlay};
use crate::RenderOptions;
use image::{imageops, DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};

pub(crate) struct MinecraftSkin(DynamicImage);

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum MinecraftSkinVersion {
	Classic, // 64x32
	Modern,  // 64x64
	Invalid,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SkinModel {
	Slim,
	Regular,
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

const SKEW_A: f32 = 26.0 / 45.0; // 0.57777777
const SKEW_B: f32 = SKEW_A * 2.0; // 1.15555555

impl MinecraftSkin {
	#[inline]
	pub fn new(skin: DynamicImage) -> MinecraftSkin {
		MinecraftSkin(skin)
	}

	#[inline]
	fn version(&self) -> MinecraftSkinVersion {
		match self.0.dimensions() {
			(64, 32) => MinecraftSkinVersion::Classic,
			(64, 64) => MinecraftSkinVersion::Modern,
			_ => MinecraftSkinVersion::Invalid,
		}
	}

	pub(crate) fn get_part(&self, layer: Layer, part: BodyPart, model: SkinModel) -> DynamicImage {
		let arm_width = match model {
			SkinModel::Slim => 3,
			SkinModel::Regular => 4,
		};

		match layer {
			Layer::Both => {
				if self.version() != MinecraftSkinVersion::Modern && part != Head {
					return self.get_part(Layer::Bottom, part, model);
				}

				let mut bottom = self.get_part(Layer::Bottom, part, model);
				let mut top = self.get_part(Layer::Top, part, model);
				apply_minecraft_transparency(&mut top);
				fast_overlay(&mut bottom, &top, 0, 0);
				bottom
			}
			Layer::Bottom => match part {
				BodyPart::Head => self.0.crop_imm(8, 8, 8, 8),
				BodyPart::Body => self.0.crop_imm(20, 20, 8, 12),
				BodyPart::ArmRight => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(36, 52, arm_width, 12),
					_ => self.get_part(Bottom, ArmLeft, model).fliph(),
				},
				BodyPart::ArmLeft => self.0.crop_imm(44, 20, arm_width, 12),
				BodyPart::LegRight => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(20, 52, 4, 12),
					_ => self.get_part(Bottom, LegLeft, model).fliph(),
				},
				BodyPart::LegLeft => self.0.crop_imm(4, 20, 4, 12),
			},
			Layer::Top => match part {
				BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
				BodyPart::Body => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(20, 36, 8, 12),
					_ => self.get_part(Bottom, Body, model),
				},
				BodyPart::ArmLeft => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(52, 52, arm_width, 12),
					_ => self.get_part(Bottom, ArmLeft, model),
				},
				BodyPart::ArmRight => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(44, 36, arm_width, 12),
					_ => self.get_part(Bottom, ArmLeft, model).fliph(),
				},
				BodyPart::LegLeft => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(4, 52, 4, 12),
					_ => self.get_part(Bottom, LegLeft, model),
				},
				BodyPart::LegRight => match self.version() {
					MinecraftSkinVersion::Modern => self.0.crop_imm(4, 36, 4, 12),
					_ => self.get_part(Bottom, LegLeft, model).fliph(),
				},
			},
		}
	}

	pub(crate) fn get_cape(&self) -> DynamicImage {
		self.0.crop_imm(1, 1, 10, 16)
	}

	pub(crate) fn render_body(&self, options: RenderOptions) -> DynamicImage {
		let layer_type = if options.armored {
			Layer::Both
		} else {
			Layer::Bottom
		};

		let img_width = match options.model {
			SkinModel::Slim => 14,
			SkinModel::Regular => 16,
		};

		let arm_width = match options.model {
			SkinModel::Slim => 3,
			SkinModel::Regular => 4,
		};

		let mut image = RgbaImage::new(img_width, 32);

		// Head (centered)
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::Head, options.model),
			arm_width,
			0,
		);
		// Body (centered)
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::Body, options.model),
			arm_width,
			8,
		);
		// Right Arm (viewer left)
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::ArmRight, options.model),
			0,
			8,
		);
		// Left Arm (viewer right)
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::ArmLeft, options.model),
			i64::from(img_width) - arm_width,
			8,
		);
		// Right Leg
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::LegLeft, options.model),
			arm_width,
			20,
		);
		// Left Leg
		imageops::overlay(
			&mut image,
			&self.get_part(layer_type, BodyPart::LegRight, options.model),
			arm_width + 4,
			20,
		);

		DynamicImage::ImageRgba8(image)
	}

	pub(crate) fn render_cube(&self, size: u32, options: RenderOptions) -> DynamicImage {
		let scale = (size as f32) / 20.0_f32;

		let x_render_offset = scale.ceil() as i64;
		let z_render_offset = x_render_offset / 2;

		let mut render = RgbaImage::new(size, size);

		let z_offset = scale * 3.0;
		let x_offset = scale * 2.0;

		let head_orig_top = self.0.crop_imm(8, 0, 8, 8);
		let head_orig_right = self.0.crop_imm(0, 8, 8, 8);
		let head_orig_front = self.0.crop_imm(8, 8, 8, 8);

		let head_orig_top_overlay = self.0.crop_imm(40, 0, 8, 8);
		let head_orig_right_overlay = self.0.crop_imm(32, 8, 8, 8);
		let head_orig_front_overlay = self.0.crop_imm(40, 8, 8, 8);

		// Shade right texture darker to show depth
		let head_orig_right = head_orig_right.brighten(-4);
		let head_orig_right_overlay = head_orig_right_overlay.brighten(-4);

		// The warp_into function clears every part of the output image that is not part of the pre-image.
		// As a workaround, we ask warp_into to draw into a scratch image, overlay the final image with the
		// scratch image, and let the scratch be overwritten.
		let mut scratch = RgbaImage::new(size, size);

		// head top
		let head_top_skew =
			Projection::from_matrix([1.0, 1.0, 0.0, -SKEW_A, SKEW_A, 0.0, 0.0, 0.0, 1.0]).unwrap()
				* Projection::translate(-0.5 - z_offset, x_offset + z_offset - 0.5)
				* Projection::scale(scale, scale + (1.0 / 8.0));
		warp_into(
			&head_orig_top.into_rgba8(),
			&head_top_skew,
			Interpolation::Nearest,
			Rgba([0, 0, 0, 0]),
			&mut scratch,
		);
		imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

		// head front
		let head_front_skew =
			Projection::from_matrix([1.0, 0.0, 0.0, -SKEW_A, SKEW_B, SKEW_A, 0.0, 0.0, 1.0])
				.unwrap() * Projection::translate(
				x_offset + 7.5 * scale - 0.5,
				(x_offset + 8.0 * scale) + z_offset - 0.5,
			) * Projection::scale(scale, scale);
		warp_into(
			&head_orig_front.into_rgba8(),
			&head_front_skew,
			Interpolation::Nearest,
			Rgba([0, 0, 0, 0]),
			&mut scratch,
		);
		imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

		// head right
		let head_right_skew =
			Projection::from_matrix([1.0, 0.0, 0.0, SKEW_A, SKEW_B, 0.0, 0.0, 0.0, 1.0]).unwrap()
				* Projection::translate(x_offset - (scale / 2.0), z_offset + scale)
				* Projection::scale(scale + (0.5 / 8.0), scale + (1.0 / 8.0));
		warp_into(
			&head_orig_right.into_rgba8(),
			&head_right_skew,
			Interpolation::Nearest,
			Rgba([0, 0, 0, 0]),
			&mut scratch,
		);
		imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

		if options.armored {
			// head top overlay
			warp_into(
				&head_orig_top_overlay.into_rgba8(),
				&head_top_skew,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

			// head front overlay
			warp_into(
				&head_orig_front_overlay.into_rgba8(),
				&head_front_skew,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

			// head right overlay
			warp_into(
				&head_orig_right_overlay.into_rgba8(),
				&head_right_skew,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);
		}

		DynamicImage::ImageRgba8(render)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::{DynamicImage, Rgba, RgbaImage};

	#[test]
	fn test_render_body_all_positions() {
		// Define unique colors for each part
		let head_color = Rgba([255, 0, 0, 255]); // Red
		let body_color = Rgba([0, 255, 0, 255]); // Green
		let left_arm_color = Rgba([0, 0, 255, 255]); // Blue
		let right_arm_color = Rgba([255, 255, 0, 255]); // Yellow
		let left_leg_color = Rgba([255, 0, 255, 255]); // Magenta
		let right_leg_color = Rgba([0, 255, 255, 255]); // Cyan

		// Create a blank modern skin (64x64)
		let mut skin = RgbaImage::new(64, 64);

		// Fill each part with its color (modern skin layout)
		// Head (8,8,8,8)
		for y in 8..16 {
			for x in 8..16 {
				skin.put_pixel(x, y, head_color);
			}
		}
		// Body (20,20,8,12)
		for y in 20..32 {
			for x in 20..28 {
				skin.put_pixel(x, y, body_color);
			}
		}
		// Left Arm (44,20,4,12)
		for y in 20..32 {
			for x in 44..48 {
				skin.put_pixel(x, y, left_arm_color);
			}
		}
		// Right Arm (36,52,4,12)
		for y in 52..64 {
			for x in 36..40 {
				skin.put_pixel(x, y, right_arm_color);
			}
		}
		// Left Leg (4,20,4,12)
		for y in 20..32 {
			for x in 4..8 {
				skin.put_pixel(x, y, left_leg_color);
			}
		}
		// Right Leg (20,52,4,12)
		for y in 52..64 {
			for x in 20..24 {
				skin.put_pixel(x, y, right_leg_color);
			}
		}

		let skin = MinecraftSkin(DynamicImage::ImageRgba8(skin));
		let options = RenderOptions {
			armored: false,
			model: SkinModel::Regular,
		};
		let rendered = skin.render_body(options).into_rgba8();

		// For Regular model: img_width = 16, arm_width = 4
		// Head: (4, 0) to (11, 7)
		assert_eq!(rendered.get_pixel(4, 0).0, head_color.0);
		assert_eq!(rendered.get_pixel(7, 3).0, head_color.0);

		// Body: (4, 8) to (11, 19)
		assert_eq!(rendered.get_pixel(4, 8).0, body_color.0);
		assert_eq!(rendered.get_pixel(7, 15).0, body_color.0);

		// Right Arm (viewer left): (0, 8) to (3, 19)
		assert_eq!(rendered.get_pixel(0, 8).0, right_arm_color.0);
		assert_eq!(rendered.get_pixel(3, 15).0, right_arm_color.0);

		// Left Arm (viewer right): (12, 8) to (15, 19)
		assert_eq!(rendered.get_pixel(12, 8).0, left_arm_color.0);
		assert_eq!(rendered.get_pixel(15, 15).0, left_arm_color.0);

		// Right Leg (viewer left): (4, 20) to (7, 31)
		assert_eq!(rendered.get_pixel(4, 20).0, left_leg_color.0); // should be left_leg_color (magenta)
		assert_eq!(rendered.get_pixel(7, 25).0, left_leg_color.0);

		// Left Leg (viewer right): (8, 20) to (11, 31)
		assert_eq!(rendered.get_pixel(8, 20).0, right_leg_color.0); // should be right_leg_color (cyan)
		assert_eq!(rendered.get_pixel(11, 25).0, right_leg_color.0);
	}
}
