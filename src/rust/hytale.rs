extern crate image;

use crate::cube::render_head_cube;
use crate::utils::fast_overlay;
use image::{imageops, DynamicImage, GenericImageView, RgbaImage};

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
				let top = self.get_part(Layer::Top, part, model);
				// No Hytale equivalent of Minecraft's legacy opaque-overlay hack.
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

	pub(crate) fn render_cube(&self, size: u32, options: RenderOptions) -> RgbaImage {
		render_head_cube(&self.0, size, options.armored)
	}
}
