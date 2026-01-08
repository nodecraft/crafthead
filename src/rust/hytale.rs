extern crate image;

use crate::utils::{apply_minecraft_transparency, fast_overlay};
use image::{imageops, DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};

/// Hytale skin handler - currently a stub with Minecraft-like layout
/// TODO: Update texture coordinates when Hytale skin format is known
pub(crate) struct HytaleSkin(DynamicImage);

#[allow(dead_code)] // Stub for future use when Hytale format is known
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum HytaleSkinVersion {
	Standard, // Assumed standard format, update when known
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

const SKEW_A: f32 = 26.0 / 45.0;
const SKEW_B: f32 = SKEW_A * 2.0;

pub(crate) struct RenderOptions {
	pub armored: bool,
	pub model: SkinModel,
}

impl HytaleSkin {
	#[inline]
	pub fn new(skin: DynamicImage) -> HytaleSkin {
		HytaleSkin(skin)
	}

	#[allow(dead_code)] // Stub for future use when Hytale format is known
	#[inline]
	fn version(&self) -> HytaleSkinVersion {
		// TODO: Update when Hytale skin format dimensions are known
		// For now, accept 64x64 as the standard format (placeholder)
		match self.0.dimensions() {
			(64, 64) => HytaleSkinVersion::Standard,
			_ => HytaleSkinVersion::Invalid,
		}
	}

	/// Get a body part from the skin texture
	/// TODO: Update texture coordinates when Hytale skin format is known
	/// Currently using Minecraft-like coordinates as a placeholder
	pub(crate) fn get_part(&self, layer: Layer, part: BodyPart, model: SkinModel) -> DynamicImage {
		let arm_width = match model {
			SkinModel::Slim => 3,
			SkinModel::Regular => 4,
		};

		match layer {
			Layer::Both => {
				let mut bottom = self.get_part(Layer::Bottom, part, model);
				let mut top = self.get_part(Layer::Top, part, model);
				apply_minecraft_transparency(&mut top);
				fast_overlay(&mut bottom, &top, 0, 0);
				bottom
			}
			// TODO: These coordinates are placeholders - update when Hytale format is known
			Layer::Bottom => match part {
				BodyPart::Head => self.0.crop_imm(8, 8, 8, 8),
				BodyPart::Body => self.0.crop_imm(20, 20, 8, 12),
				BodyPart::ArmRight => self.0.crop_imm(36, 52, arm_width, 12),
				BodyPart::ArmLeft => self.0.crop_imm(44, 20, arm_width, 12),
				BodyPart::LegRight => self.0.crop_imm(20, 52, 4, 12),
				BodyPart::LegLeft => self.0.crop_imm(4, 20, 4, 12),
			},
			Layer::Top => match part {
				BodyPart::Head => self.0.crop_imm(40, 8, 8, 8),
				BodyPart::Body => self.0.crop_imm(20, 36, 8, 12),
				BodyPart::ArmLeft => self.0.crop_imm(52, 52, arm_width, 12),
				BodyPart::ArmRight => self.0.crop_imm(44, 36, arm_width, 12),
				BodyPart::LegLeft => self.0.crop_imm(4, 52, 4, 12),
				BodyPart::LegRight => self.0.crop_imm(4, 36, 4, 12),
			},
		}
	}

	pub(crate) fn get_cape(&self) -> DynamicImage {
		// TODO: Update cape coordinates when Hytale format is known
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

		// TODO: Update these coordinates when Hytale format is known
		let head_orig_top = self.0.crop_imm(8, 0, 8, 8);
		let head_orig_right = self.0.crop_imm(0, 8, 8, 8);
		let head_orig_front = self.0.crop_imm(8, 8, 8, 8);

		let head_orig_top_overlay = self.0.crop_imm(40, 0, 8, 8);
		let head_orig_right_overlay = self.0.crop_imm(32, 8, 8, 8);
		let head_orig_front_overlay = self.0.crop_imm(40, 8, 8, 8);

		let head_orig_right = head_orig_right.brighten(-4);
		let head_orig_right_overlay = head_orig_right_overlay.brighten(-4);

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
			warp_into(
				&head_orig_top_overlay.into_rgba8(),
				&head_top_skew,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

			warp_into(
				&head_orig_front_overlay.into_rgba8(),
				&head_front_skew,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);

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
