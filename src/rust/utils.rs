use cfg_if::cfg_if;
use image::{imageops, DynamicImage, GenericImage, GenericImageView};

cfg_if! {
	// When the `console_error_panic_hook` feature is enabled, we can call the
	// `set_panic_hook` function at least once during initialization, and then
	// we will get better error messages if our code ever panics.
	//
	// For more details see
	// https://github.com/rustwasm/console_error_panic_hook#readme
	if #[cfg(feature = "console_error_panic_hook")] {
		extern crate console_error_panic_hook;
		pub use self::console_error_panic_hook::set_once as set_panic_hook;
	} else {
		#[allow(dead_code)]
		#[inline]
		pub fn set_panic_hook() {}
	}
}

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

pub(crate) fn apply_minecraft_transparency(img: &mut DynamicImage) {
	let (width, height) = img.dimensions();
	apply_minecraft_transparency_region(img, 0, 0, width, height);
}

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

pub(crate) fn fast_overlay(bottom: &mut DynamicImage, top: &DynamicImage, x: u32, y: u32) {
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
