//! Post-processing effects for rendered images

use image::RgbaImage;

/// Apply a simple box blur for post-processing anti-aliasing
///
/// This creates a softer appearance that matches in-game rendering.
/// The blur_amount controls the intensity (0.0 = no blur, 1.0 = full blur).
///
/// # Arguments
///
/// * `image` - The image to blur (modified in-place)
/// * `blur_amount` - The blur intensity (0.0 to 1.0)
pub(crate) fn apply_blur(image: &mut RgbaImage, blur_amount: f32) {
	if blur_amount <= 0.0 {
		return;
	}

	let width = image.width();
	let height = image.height();
	let mut blurred = image.clone();

	// Simple 3x3 box blur
	let radius = 1i32;
	let weight = blur_amount.min(1.0);

	for y in (radius as u32)..(height - radius as u32) {
		for x in (radius as u32)..(width - radius as u32) {
			let mut r_sum = 0u32;
			let mut g_sum = 0u32;
			let mut b_sum = 0u32;
			let mut a_sum = 0u32;
			let mut count = 0u32;

			// Sample 3x3 neighborhood
			for dy in -radius..=radius {
				for dx in -radius..=radius {
					let px = ((x as i32) + dx).max(0).min((width - 1) as i32) as u32;
					let py = ((y as i32) + dy).max(0).min((height - 1) as i32) as u32;
					let pixel = image.get_pixel(px, py);
					r_sum += pixel[0] as u32;
					g_sum += pixel[1] as u32;
					b_sum += pixel[2] as u32;
					a_sum += pixel[3] as u32;
					count += 1;
				}
			}

			// Blend original with blurred
			let original = image.get_pixel(x, y);
			let blurred_pixel = image::Rgba([
				(r_sum / count) as u8,
				(g_sum / count) as u8,
				(b_sum / count) as u8,
				(a_sum / count) as u8,
			]);

			let blended = image::Rgba([
				((original[0] as f32 * (1.0 - weight)) + (blurred_pixel[0] as f32 * weight)) as u8,
				((original[1] as f32 * (1.0 - weight)) + (blurred_pixel[1] as f32 * weight)) as u8,
				((original[2] as f32 * (1.0 - weight)) + (blurred_pixel[2] as f32 * weight)) as u8,
				original[3], // Preserve alpha
			]);

			blurred.put_pixel(x, y, blended);
		}
	}

	*image = blurred;
}
