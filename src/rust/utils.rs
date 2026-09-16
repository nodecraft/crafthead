use cfg_if::cfg_if;
use image::{imageops, DynamicImage, GenericImage, GenericImageView, Rgba};

cfg_if! {
	// When the `console_error_panic_hook` feature is enabled, we can call the
	// `set_panic_hook` function at least once during initialization, and then
	// we will get better error messages if our code ever panics.
	//
	// For more details see
	// https://github.com/rustwasm/console_error_panic_hook#readme
	if #[cfg(feature = "console_error_panic_hook")] {
		extern crate console_error_panic_hook;
		#[allow(unused_imports)]
		pub use self::console_error_panic_hook::set_once as set_panic_hook;
	} else {
		#[allow(dead_code)]
		#[inline]
		pub fn set_panic_hook() {}
	}
}

#[inline]
fn is_image_region_transparent_to_minecraft(
	img: &DynamicImage,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
) -> bool {
	// This is based on ImageBufferDownload from Minecraft Beta 1.7.3. It seems that this code hasn't
	// changed at all since then, and I hope it doesn't change...
	for cy in y..y + height {
		for cx in x..x + width {
			let p = img.get_pixel(cx, cy);
			if p[3] < 128 {
				return true;
			}
		}
	}
	false
}

/// Discards an overlay layer that is fully opaque. Legacy 64x32 skins left junk in the overlay
/// region, so Minecraft treats "no transparency at all" as "nothing was drawn here".
#[inline]
pub(crate) fn apply_minecraft_transparency(img: &mut DynamicImage) {
	let (width, height) = img.dimensions();
	apply_minecraft_transparency_region(img, 0, 0, width, height);
}

#[inline]
fn apply_minecraft_transparency_region(
	img: &mut DynamicImage,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
) {
	if is_image_region_transparent_to_minecraft(img, x, y, width, height) {
		return;
	}

	for cy in y..y + height {
		for cx in x..x + width {
			let mut p = img.get_pixel(cx, cy);
			p[3] = 0x00;
			img.put_pixel(cx, cy, p);
		}
	}
}

/// Source-over compositing. Done in integer space rather than via `Pixel::blend`, whose float
/// rounding leaves an opaque result sitting at alpha 254.
#[inline]
fn blend_over(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
	let top_alpha = u32::from(top[3]);
	// What the base still contributes once the top has been laid over it.
	let bottom_alpha = u32::from(bottom[3]) * (255 - top_alpha) / 255;
	let out_alpha = top_alpha + bottom_alpha;
	if out_alpha == 0 {
		return Rgba([0, 0, 0, 0]);
	}

	let mut out = [0_u8; 4];
	for channel in 0..3 {
		let weighted =
			u32::from(top[channel]) * top_alpha + u32::from(bottom[channel]) * bottom_alpha;
		out[channel] = (weighted / out_alpha) as u8;
	}
	out[3] = out_alpha as u8;
	Rgba(out)
}

#[inline]
pub(crate) fn fast_overlay(bottom: &mut DynamicImage, top: &DynamicImage, x: u32, y: u32) {
	// Based on https://github.com/minotar/imgd/blob/master/process.go#L386, which predates the
	// translucent outer layers Minecraft added in 1.8 and so forced every pixel opaque.
	let bottom_dims = bottom.dimensions();
	let top_dims = top.dimensions();

	// Crop our top image if we're going out of bounds
	let (range_width, range_height) = imageops::overlay_bounds(bottom_dims, top_dims, x, y);

	for top_y in 0..range_height {
		for top_x in 0..range_width {
			let p = top.get_pixel(top_x, top_y);
			if p[3] == 0 {
				continue;
			}

			// Blend rather than replace, so a translucent outer skin layer keeps its opacity.
			let blended = blend_over(bottom.get_pixel(x + top_x, y + top_y), p);
			bottom.put_pixel(x + top_x, y + top_y, blended);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::{Rgba, RgbaImage};

	#[test]
	fn test_is_image_region_transparent_to_minecraft_transparent() {
		let mut img = RgbaImage::new(10, 10);
		img.put_pixel(5, 5, Rgba([0, 0, 0, 127])); // Transparent pixel
		let img = DynamicImage::ImageRgba8(img);
		assert!(is_image_region_transparent_to_minecraft(&img, 0, 0, 10, 10));
	}

	#[test]
	fn test_is_image_region_transparent_to_minecraft_opaque() {
		let mut img = RgbaImage::new(10, 10);
		for y in 0..10 {
			for x in 0..10 {
				img.put_pixel(x, y, Rgba([0, 0, 0, 255])); // Fully opaque
			}
		}
		let img = DynamicImage::ImageRgba8(img);
		assert!(!is_image_region_transparent_to_minecraft(
			&img, 0, 0, 10, 10
		));
	}

	#[test]
	fn test_apply_minecraft_transparency() {
		let mut img = RgbaImage::new(10, 10);
		img.put_pixel(5, 5, Rgba([0, 0, 0, 127])); // Transparent pixel
		let mut img = DynamicImage::ImageRgba8(img);
		apply_minecraft_transparency(&mut img);
		assert_eq!(img.get_pixel(5, 5)[3], 127); // Should remain transparent
		assert_eq!(img.get_pixel(0, 0)[3], 0); // Should be made transparent
	}

	#[test]
	fn test_apply_minecraft_transparency_region() {
		let mut img = RgbaImage::new(10, 10);
		img.put_pixel(5, 5, Rgba([0, 0, 0, 127])); // Transparent pixel
		let mut img = DynamicImage::ImageRgba8(img);
		apply_minecraft_transparency_region(&mut img, 0, 0, 10, 10);
		assert_eq!(img.get_pixel(5, 5)[3], 127); // Should remain transparent
		assert_eq!(img.get_pixel(0, 0)[3], 0); // Should be made transparent
	}

	#[test]
	fn test_apply_minecraft_transparency_fully_transparent() {
		let mut img = RgbaImage::new(10, 10);
		for y in 0..10 {
			for x in 0..10 {
				img.put_pixel(x, y, Rgba([0, 0, 0, 0])); // Fully transparent
			}
		}
		let mut img = DynamicImage::ImageRgba8(img);
		apply_minecraft_transparency(&mut img);
		assert_eq!(img.get_pixel(0, 0)[3], 0); // Should remain transparent
	}

	#[test]
	fn test_apply_minecraft_transparency_region_fully_transparent() {
		let mut img = RgbaImage::new(10, 10);
		for y in 0..10 {
			for x in 0..10 {
				img.put_pixel(x, y, Rgba([0, 0, 0, 0])); // Fully transparent
			}
		}
		let mut img = DynamicImage::ImageRgba8(img);
		apply_minecraft_transparency_region(&mut img, 0, 0, 10, 10);
		assert_eq!(img.get_pixel(0, 0)[3], 0); // Should remain transparent
	}

	#[test]
	fn test_fast_overlay_within_bounds() {
		let bottom = RgbaImage::new(10, 10);
		let mut top = RgbaImage::new(5, 5);
		for y in 0..5 {
			for x in 0..5 {
				top.put_pixel(x, y, Rgba([255, 0, 0, 255])); // Red opaque pixel
			}
		}
		let mut bottom = DynamicImage::ImageRgba8(bottom);
		let top = DynamicImage::ImageRgba8(top);
		fast_overlay(&mut bottom, &top, 2, 2);
		assert_eq!(bottom.get_pixel(2, 2), Rgba([255, 0, 0, 255])); // Should be red
	}

	#[test]
	fn test_fast_overlay_out_of_bounds() {
		let bottom = RgbaImage::new(10, 10);
		let mut top = RgbaImage::new(5, 5);
		for y in 0..5 {
			for x in 0..5 {
				top.put_pixel(x, y, Rgba([255, 0, 0, 255])); // Red opaque pixel
			}
		}
		let mut bottom = DynamicImage::ImageRgba8(bottom);
		let top = DynamicImage::ImageRgba8(top);
		fast_overlay(&mut bottom, &top, 8, 8);
		assert_eq!(bottom.get_pixel(8, 8), Rgba([255, 0, 0, 255])); // Should be red
		assert_eq!(bottom.get_pixel(9, 9), Rgba([255, 0, 0, 255])); // Should be red
		assert_eq!(bottom.get_pixel(7, 7), Rgba([0, 0, 0, 0])); // Should be unchanged
	}

	#[test]
	fn test_fast_overlay_fully_transparent_top() {
		let bottom = RgbaImage::new(10, 10);
		let mut top = RgbaImage::new(5, 5);
		for y in 0..5 {
			for x in 0..5 {
				top.put_pixel(x, y, Rgba([255, 0, 0, 0])); // Fully transparent pixel
			}
		}
		let mut bottom = DynamicImage::ImageRgba8(bottom);
		let top = DynamicImage::ImageRgba8(top);
		fast_overlay(&mut bottom, &top, 2, 2);
		assert_eq!(bottom.get_pixel(2, 2), Rgba([0, 0, 0, 0])); // Should remain unchanged
	}

	#[test]
	fn test_fast_overlay_blends_translucent_top() {
		let mut bottom = RgbaImage::new(2, 2);
		let mut top = RgbaImage::new(2, 2);
		for y in 0..2 {
			for x in 0..2 {
				bottom.put_pixel(x, y, Rgba([0, 0, 255, 255])); // Opaque blue
				top.put_pixel(x, y, Rgba([255, 0, 0, 64])); // 25% red
			}
		}
		let mut bottom = DynamicImage::ImageRgba8(bottom);
		let top = DynamicImage::ImageRgba8(top);
		fast_overlay(&mut bottom, &top, 0, 0);
		assert_eq!(bottom.get_pixel(0, 0), Rgba([64, 0, 191, 255])); // Blended, not solid red
	}

	#[test]
	fn test_fast_overlay_translucent_top_over_nothing() {
		let bottom = RgbaImage::new(2, 2);
		let mut top = RgbaImage::new(2, 2);
		for y in 0..2 {
			for x in 0..2 {
				top.put_pixel(x, y, Rgba([255, 0, 0, 64]));
			}
		}
		let mut bottom = DynamicImage::ImageRgba8(bottom);
		let top = DynamicImage::ImageRgba8(top);
		fast_overlay(&mut bottom, &top, 0, 0);
		assert_eq!(bottom.get_pixel(0, 0), Rgba([255, 0, 0, 64])); // Keeps its own opacity
	}
}
