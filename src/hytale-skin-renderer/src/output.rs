//! Image export functionality (PNG and GIF)

use crate::error::{Error, Result};
use image::{ImageFormat, RgbaImage};
use std::path::Path;

#[cfg(feature = "gif")]
use image::{codecs::gif::GifEncoder, Frame};
#[cfg(feature = "gif")]
use std::fs::File;

/// Export an RGBA image to a PNG file
pub fn export_png(image: &RgbaImage, path: &Path) -> Result<()> {
	image
		.save_with_format(path, ImageFormat::Png)
		.map_err(|e| Error::Image(e))
}

/// Export an RGBA image to PNG bytes
pub fn export_png_bytes(image: &RgbaImage) -> Result<Vec<u8>> {
	let mut buffer = Vec::new();
	let mut cursor = std::io::Cursor::new(&mut buffer);
	image
		.write_to(&mut cursor, ImageFormat::Png)
		.map_err(|e| Error::Image(e))?;
	Ok(buffer)
}

/// Create an RGBA image buffer of the specified size
pub fn create_image_buffer(width: u32, height: u32) -> RgbaImage {
	RgbaImage::new(width, height)
}

/// Export a sequence of RGBA images as an animated GIF
///
/// # Arguments
/// * `frames` - Slice of RGBA images to include in the animation
/// * `path` - Output path for the GIF file
/// * `frame_delay_ms` - Delay between frames in milliseconds (e.g., 33 for ~30fps, 16 for ~60fps)
///
/// # Returns
/// Result indicating success or failure
#[cfg(feature = "gif")]
pub fn export_gif(frames: &[RgbaImage], path: &Path, frame_delay_ms: u16) -> Result<()> {
	if frames.is_empty() {
		return Err(Error::Parse("No frames provided for GIF".to_string()));
	}

	let file = File::create(path).map_err(|e| Error::Io(e))?;
	let mut encoder = GifEncoder::new(file);

	// Set repeat to infinite loop
	encoder
		.set_repeat(image::codecs::gif::Repeat::Infinite)
		.map_err(|e| Error::Image(e))?;

	// Convert frame delay from ms to centiseconds (GIF uses 10ms units)
	let delay_centisecs = (frame_delay_ms / 10).max(1) as u32;

	for frame_image in frames {
		// Create a frame with the specified delay
		let frame = Frame::from_parts(
			frame_image.clone(),
			0,
			0,
			image::Delay::from_numer_denom_ms(delay_centisecs * 10, 1),
		);
		encoder.encode_frame(frame).map_err(|e| Error::Image(e))?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use image::Rgba;
	use std::fs;

	#[test]
	fn test_create_rgba_buffer() {
		let image = create_image_buffer(100, 200);
		assert_eq!(image.width(), 100);
		assert_eq!(image.height(), 200);
	}

	#[test]
	fn test_write_valid_png_file() {
		let mut image = create_image_buffer(64, 64);
		// Fill with test pattern
		for y in 0..64 {
			for x in 0..64 {
				let pixel = Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255]);
				image.put_pixel(x, y, pixel);
			}
		}

		let test_path = Path::new("test_output.png");
		let result = export_png(&image, test_path);
		assert!(result.is_ok());

		// Clean up
		let _ = fs::remove_file(test_path);
	}

	#[test]
	fn test_png_file_is_readable() {
		let mut image = create_image_buffer(32, 32);
		// Fill with a simple pattern
		for y in 0..32 {
			for x in 0..32 {
				let pixel = if (x + y) % 2 == 0 {
					Rgba([255, 255, 255, 255])
				} else {
					Rgba([0, 0, 0, 255])
				};
				image.put_pixel(x, y, pixel);
			}
		}

		let test_path = Path::new("test_readable.png");
		export_png(&image, test_path).unwrap();

		// Try to read it back
		let loaded = image::open(test_path);
		assert!(loaded.is_ok());
		let loaded_image = loaded.unwrap();
		assert_eq!(loaded_image.width(), 32);
		assert_eq!(loaded_image.height(), 32);

		// Clean up
		let _ = fs::remove_file(test_path);
	}

	#[test]
	fn test_output_dimensions_match() {
		let width = 256;
		let height = 128;
		let image = create_image_buffer(width, height);

		let test_path = Path::new("test_dimensions.png");
		export_png(&image, test_path).unwrap();

		let loaded = image::open(test_path).unwrap();
		assert_eq!(loaded.width(), width);
		assert_eq!(loaded.height(), height);

		// Clean up
		let _ = fs::remove_file(test_path);
	}

	#[test]
	fn test_export_png_bytes() {
		let image = create_image_buffer(16, 16);
		let result = export_png_bytes(&image);

		assert!(result.is_ok());
		let bytes = result.unwrap();
		assert!(!bytes.is_empty());

		// Should be valid PNG (starts with PNG signature)
		assert!(bytes.len() > 8);
		assert_eq!(
			&bytes[0..8],
			&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
		);
	}
}
