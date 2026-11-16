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

/// The colours of an image that has few enough of them to be stored as palette indices.
struct Palette {
	/// One palette index per pixel
	indices: Vec<u8>,
	/// RGB triplets, in palette order
	rgb: Vec<u8>,
	/// Alpha per palette entry, with the trailing opaque entries left off
	alpha: Vec<u8>,
}

impl Palette {
	const MAX_COLOURS: usize = 256;
	/// Open-addressing table size; a power of two kept at most half full so probes stay short.
	const SLOTS: usize = 512;

	/// Returns None as soon as the image turns out to have more colours than a palette can hold.
	fn from_image(image: &RgbaImage) -> Option<Self> {
		let mut colours: Vec<u32> = Vec::with_capacity(Self::MAX_COLOURS);
		// Slot value is palette index + 1, so 0 means empty.
		let mut slots = [0u16; Self::SLOTS];
		let mut indices = Vec::with_capacity(image.as_raw().len() / 4);

		// Renders are mostly flat runs, so remembering the previous pixel skips most lookups.
		let mut previous: Option<(u32, u8)> = None;
		for pixel in image.as_raw().chunks_exact(4) {
			let colour = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
			if let Some((previous_colour, previous_index)) = previous {
				if previous_colour == colour {
					indices.push(previous_index);
					continue;
				}
			}

			let mut slot = (colour.wrapping_mul(0x9E37_79B1) >> 23) as usize;
			let index = loop {
				let entry = slots[slot];
				if entry == 0 {
					if colours.len() == Self::MAX_COLOURS {
						return None;
					}
					colours.push(colour);
					slots[slot] = colours.len() as u16;
					break (colours.len() - 1) as u8;
				}
				if colours[entry as usize - 1] == colour {
					break (entry - 1) as u8;
				}
				slot = (slot + 1) % Self::SLOTS;
			};
			previous = Some((colour, index));
			indices.push(index);
		}

		let mut rgb = Vec::with_capacity(colours.len() * 3);
		let mut alpha = Vec::with_capacity(colours.len());
		for colour in &colours {
			let [r, g, b, a] = colour.to_le_bytes();
			rgb.extend_from_slice(&[r, g, b]);
			alpha.push(a);
		}
		while alpha.last() == Some(&u8::MAX) {
			alpha.pop();
		}

		Some(Palette {
			indices,
			rgb,
			alpha,
		})
	}
}

/// Export an RGBA image to PNG bytes
///
/// The decoded pixels always equal `image`, but images with at most 256 colours (most
/// pixel-art renders) are stored with a palette, which is smaller and much faster to encode.
pub fn export_png_bytes(image: &RgbaImage) -> Result<Vec<u8>> {
	let (width, height) = image.dimensions();
	let palette = Palette::from_image(image);

	// These renders compress well; a quarter of the raw size avoids most regrowth.
	let mut buffer = Vec::with_capacity((image.as_raw().len() / 4).max(1024));
	let mut encoder = png::Encoder::new(&mut buffer, width, height);
	encoder.set_depth(png::BitDepth::Eight);
	encoder.set_compression(png::Compression::Fast);
	// Trying every filter per row (the default) costs about twice the time for no size gain here.
	encoder.set_filter(png::Filter::Paeth);

	let pixels = if let Some(palette) = &palette {
		encoder.set_color(png::ColorType::Indexed);
		encoder.set_palette(palette.rgb.as_slice());
		if !palette.alpha.is_empty() {
			encoder.set_trns(palette.alpha.as_slice());
		}
		palette.indices.as_slice()
	} else {
		encoder.set_color(png::ColorType::Rgba);
		image.as_raw().as_slice()
	};

	let encoding_error = |e: png::EncodingError| Error::InvalidData(format!("PNG encoding: {}", e));
	let mut writer = encoder.write_header().map_err(encoding_error)?;
	writer.write_image_data(pixels).map_err(encoding_error)?;
	writer.finish().map_err(encoding_error)?;
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

	/// PNG colour type byte of the IHDR chunk: 3 is palette, 6 is RGBA.
	fn png_colour_type(png: &[u8]) -> u8 {
		png[25]
	}

	fn decode(png: &[u8]) -> RgbaImage {
		image::load_from_memory(png).unwrap().into_rgba8()
	}

	#[test]
	fn test_few_colours_round_trip_through_a_palette() {
		// Includes translucent pixels and two fully transparent ones that differ only in RGB.
		let colours = [
			Rgba([255, 0, 0, 255]),
			Rgba([0, 0, 0, 0]),
			Rgba([12, 34, 56, 0]),
			Rgba([0, 255, 0, 128]),
			Rgba([0, 0, 255, 255]),
			Rgba([255, 255, 255, 1]),
		];
		let image = RgbaImage::from_fn(37, 23, |x, y| colours[((x / 3 + y * 5) % 6) as usize]);

		let png = export_png_bytes(&image).unwrap();
		assert_eq!(png_colour_type(&png), 3);
		assert!(decode(&png) == image);
	}

	#[test]
	fn test_palette_holds_exactly_256_colours() {
		let image = RgbaImage::from_fn(16, 16, |x, y| Rgba([(x + y * 16) as u8, 7, 9, 255]));

		let png = export_png_bytes(&image).unwrap();
		assert_eq!(png_colour_type(&png), 3);
		assert!(decode(&png) == image);
	}

	#[test]
	fn test_many_colours_round_trip_as_rgba() {
		// One colour too many for a palette, with varying alpha.
		let image = RgbaImage::from_fn(257, 1, |x, _| {
			Rgba([(x % 256) as u8, (x / 256) as u8, 200, (x % 251) as u8])
		});

		let png = export_png_bytes(&image).unwrap();
		assert_eq!(png_colour_type(&png), 6);
		assert!(decode(&png) == image);
	}

	#[test]
	fn test_opaque_palette_image_round_trips() {
		// With every colour opaque no transparency chunk is written at all.
		let image = RgbaImage::from_fn(8, 8, |x, y| Rgba([(x * 30) as u8, (y * 30) as u8, 0, 255]));

		let png = export_png_bytes(&image).unwrap();
		assert_eq!(png_colour_type(&png), 3);
		assert!(decode(&png) == image);
	}
}
