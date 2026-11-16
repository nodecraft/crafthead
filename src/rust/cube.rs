//! Isometric head ("cube") rendering, shared by every skin format with a Minecraft-style head layout.

use image::{DynamicImage, Pixel, RgbaImage};
use imageproc::geometric_transformations::Projection;

const SKEW_A: f32 = 26.0 / 45.0; // 0.57777777
const SKEW_B: f32 = SKEW_A * 2.0; // 1.15555555

/// How far outside a face's projected outline to look for covered pixels.
/// The outline comes from the forward mapping while sampling uses the inverse, and the two
/// only agree to within float error.
const OUTLINE_PADDING: f32 = 2.0;

/// Render the head of `skin` as an isometric cube showing its top, front and right faces.
pub(crate) fn render_head_cube(skin: &DynamicImage, size: u32, armored: bool) -> RgbaImage {
	let scale = (size as f32) / 20.0_f32;

	let x_render_offset = scale.ceil() as u32;
	let z_render_offset = x_render_offset / 2;

	let z_offset = scale * 3.0;
	let x_offset = scale * 2.0;

	let head_top_skew =
		Projection::from_matrix([1.0, 1.0, 0.0, -SKEW_A, SKEW_A, 0.0, 0.0, 0.0, 1.0]).unwrap()
			* Projection::translate(-0.5 - z_offset, x_offset + z_offset - 0.5)
			* Projection::scale(scale, scale + (1.0 / 8.0));

	let head_front_skew =
		Projection::from_matrix([1.0, 0.0, 0.0, -SKEW_A, SKEW_B, SKEW_A, 0.0, 0.0, 1.0]).unwrap()
			* Projection::translate(
				x_offset + 7.5 * scale - 0.5,
				(x_offset + 8.0 * scale) + z_offset - 0.5,
			)
			* Projection::scale(scale, scale);

	let head_right_skew =
		Projection::from_matrix([1.0, 0.0, 0.0, SKEW_A, SKEW_B, 0.0, 0.0, 0.0, 1.0]).unwrap()
			* Projection::translate(x_offset - (scale / 2.0), z_offset + scale)
			* Projection::scale(scale + (0.5 / 8.0), scale + (1.0 / 8.0));

	let mut render = RgbaImage::new(size, size);
	let mut draw = |face: DynamicImage, projection: &Projection| {
		draw_warped_face(
			&mut render,
			&face.into_rgba8(),
			projection,
			x_render_offset,
			z_render_offset,
		);
	};

	draw(skin.crop_imm(8, 0, 8, 8), &head_top_skew);
	draw(skin.crop_imm(8, 8, 8, 8), &head_front_skew);
	// Shade right texture darker to show depth
	draw(skin.crop_imm(0, 8, 8, 8).brighten(-4), &head_right_skew);

	if armored {
		draw(skin.crop_imm(40, 0, 8, 8), &head_top_skew);
		draw(skin.crop_imm(40, 8, 8, 8), &head_front_skew);
		draw(skin.crop_imm(32, 8, 8, 8).brighten(-4), &head_right_skew);
	}

	render
}

/// Warp `face` by an affine `projection` and blend it onto `render`, shifted by the given offset.
///
/// Gives the same pixels as `warp_into` a canvas-sized scratch image followed by
/// `imageops::overlay`, but only visits the pixels the face can land on rather than the whole
/// canvas twice per face.
fn draw_warped_face(
	render: &mut RgbaImage,
	face: &RgbaImage,
	projection: &Projection,
	offset_x: u32,
	offset_y: u32,
) {
	let (face_width, face_height) = face.dimensions();
	// Pixels pushed past the canvas edge by the offset were cropped by the overlay.
	let visible_width = render.width().saturating_sub(offset_x);
	let visible_height = render.height().saturating_sub(offset_y);

	// Nearest sampling accepts source coordinates half a pixel beyond the first and last texel.
	let (right, bottom) = (face_width as f32 - 0.5, face_height as f32 - 0.5);
	let outline = [(-0.5, -0.5), (right, -0.5), (-0.5, bottom), (right, bottom)]
		.map(|corner| *projection * corner);
	let covered = |positions: [f32; 4], limit: u32| {
		let first = positions.iter().copied().fold(f32::INFINITY, f32::min) - OUTLINE_PADDING;
		let last = positions.iter().copied().fold(f32::NEG_INFINITY, f32::max) + OUTLINE_PADDING;
		let start = first.floor().max(0.0) as u32;
		let end = (last.ceil().max(0.0) as u32).saturating_add(1).min(limit);
		start..end
	};
	let columns = covered(outline.map(|(x, _)| x), visible_width);
	let rows = covered(outline.map(|(_, y)| y), visible_height);

	let inverse = projection.invert();
	for y in rows {
		for x in columns.clone() {
			let (source_x, source_y) = inverse * (x as f32, y as f32);
			if source_x < -0.5 || source_y < -0.5 {
				continue;
			}
			let (texel_x, texel_y) = ((source_x + 0.5) as u32, (source_y + 0.5) as u32);
			if texel_x >= face_width || texel_y >= face_height {
				continue;
			}
			render
				.get_pixel_mut(x + offset_x, y + offset_y)
				.blend(face.get_pixel(texel_x, texel_y));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::{imageops, Rgba};
	use imageproc::geometric_transformations::{warp_into, Interpolation};

	/// The original cube renderer: every face is warped onto a full-size scratch canvas, which is
	/// then overlaid on the render. Slow, but entirely library code, so it defines correct output.
	fn render_head_cube_reference(skin: &DynamicImage, size: u32, armored: bool) -> RgbaImage {
		let scale = (size as f32) / 20.0_f32;

		let x_render_offset = scale.ceil() as i64;
		let z_render_offset = x_render_offset / 2;

		let mut render = RgbaImage::new(size, size);

		let z_offset = scale * 3.0;
		let x_offset = scale * 2.0;

		let head_orig_top = skin.crop_imm(8, 0, 8, 8);
		let head_orig_right = skin.crop_imm(0, 8, 8, 8).brighten(-4);
		let head_orig_front = skin.crop_imm(8, 8, 8, 8);

		let head_orig_top_overlay = skin.crop_imm(40, 0, 8, 8);
		let head_orig_right_overlay = skin.crop_imm(32, 8, 8, 8).brighten(-4);
		let head_orig_front_overlay = skin.crop_imm(40, 8, 8, 8);

		let mut scratch = RgbaImage::new(size, size);

		let head_top_skew =
			Projection::from_matrix([1.0, 1.0, 0.0, -SKEW_A, SKEW_A, 0.0, 0.0, 0.0, 1.0]).unwrap()
				* Projection::translate(-0.5 - z_offset, x_offset + z_offset - 0.5)
				* Projection::scale(scale, scale + (1.0 / 8.0));
		let head_front_skew =
			Projection::from_matrix([1.0, 0.0, 0.0, -SKEW_A, SKEW_B, SKEW_A, 0.0, 0.0, 1.0])
				.unwrap()
				* Projection::translate(
					x_offset + 7.5 * scale - 0.5,
					(x_offset + 8.0 * scale) + z_offset - 0.5,
				)
				* Projection::scale(scale, scale);
		let head_right_skew =
			Projection::from_matrix([1.0, 0.0, 0.0, SKEW_A, SKEW_B, 0.0, 0.0, 0.0, 1.0]).unwrap()
				* Projection::translate(x_offset - (scale / 2.0), z_offset + scale)
				* Projection::scale(scale + (0.5 / 8.0), scale + (1.0 / 8.0));

		let mut faces = vec![
			(head_orig_top, head_top_skew),
			(head_orig_front, head_front_skew),
			(head_orig_right, head_right_skew),
		];
		if armored {
			faces.push((head_orig_top_overlay, head_top_skew));
			faces.push((head_orig_front_overlay, head_front_skew));
			faces.push((head_orig_right_overlay, head_right_skew));
		}
		for (face, projection) in faces {
			warp_into(
				&face.into_rgba8(),
				&projection,
				Interpolation::Nearest,
				Rgba([0, 0, 0, 0]),
				&mut scratch,
			);
			imageops::overlay(&mut render, &scratch, x_render_offset, z_render_offset);
		}

		render
	}

	/// A skin where every pixel differs, with translucent and transparent overlay texels so the
	/// blending of the armored faces is exercised too.
	fn noisy_skin(width: u32, height: u32) -> DynamicImage {
		let mut state = 0x2545_F491_u32;
		let mut next = || {
			state ^= state << 13;
			state ^= state >> 17;
			state ^= state << 5;
			state
		};
		let mut skin = RgbaImage::new(width, height);
		for (x, _, pixel) in skin.enumerate_pixels_mut() {
			let [r, g, b, a] = next().to_le_bytes();
			let alpha = if x < 32 {
				255
			} else {
				match a % 4 {
					0 => 0,
					1 => 255,
					_ => a,
				}
			};
			*pixel = Rgba([r, g, b, alpha]);
		}
		DynamicImage::ImageRgba8(skin)
	}

	#[test]
	fn cube_matches_reference_renderer() {
		let skin = noisy_skin(64, 64);
		// Every size the worker can ask for at the small end, then a spread up to the maximum.
		let sizes = (8..=48)
			.chain((51..=300).step_by(17))
			.chain([180, 299, 300]);
		for size in sizes {
			for armored in [false, true] {
				assert!(
					render_head_cube(&skin, size, armored)
						== render_head_cube_reference(&skin, size, armored),
					"cube differs at size {} (armored: {})",
					size,
					armored
				);
			}
		}
	}

	#[test]
	fn cube_matches_reference_for_opaque_rgb_skin() {
		// Skins without an alpha channel take the same path with every texel opaque.
		let skin = DynamicImage::ImageRgb8(noisy_skin(64, 32).into_rgb8());
		for size in [8, 20, 64, 180] {
			assert!(
				render_head_cube(&skin, size, true)
					== render_head_cube_reference(&skin, size, true),
				"cube differs at size {}",
				size
			);
		}
	}

	#[test]
	fn cube_matches_reference_for_truncated_skin() {
		// Too short for full side faces, so those crops come out 8x4 instead of 8x8.
		let skin = noisy_skin(48, 12);
		assert!(render_head_cube(&skin, 64, true) == render_head_cube_reference(&skin, 64, true));
	}
}
