//! Texture loading, UV coordinate mapping, and tint gradient support

use crate::error::Result;
use crate::models::{UvAngle, UvFace, UvOffset};
use image::{DynamicImage, GenericImageView, Rgba};
use std::path::Path;

/// A loaded texture with dimensions
#[derive(Debug, Clone)]
pub struct Texture {
	image: DynamicImage,
	width: u32,
	height: u32,
}

impl Texture {
	/// Load a texture from a file path
	pub fn from_file(path: &std::path::Path) -> Result<Self> {
		let image = image::open(path)?;
		let (width, height) = image.dimensions();
		Ok(Texture {
			image,
			width,
			height,
		})
	}

	/// Load a texture from memory (PNG bytes)
	pub fn from_bytes(data: &[u8]) -> Result<Self> {
		let image = image::load_from_memory(data)?;
		let (width, height) = image.dimensions();
		Ok(Texture {
			image,
			width,
			height,
		})
	}

	/// Get texture dimensions
	pub fn dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	/// Sample a pixel directly using absolute pixel coordinates
	/// This avoids precision loss from UV normalization
	pub fn sample_pixel(&self, x: f32, y: f32) -> Rgba<u8> {
		// Use floor() to get the pixel index, clamped to valid range
		let px = (x.floor() as i32).clamp(0, self.width as i32 - 1) as u32;
		let py = (y.floor() as i32).clamp(0, self.height as i32 - 1) as u32;
		self.image.get_pixel(px, py).clone()
	}

	/// Sample a pixel at UV coordinates (0.0 to 1.0 range)
	pub fn sample_uv(&self, u: f32, v: f32) -> Rgba<u8> {
		// Clamp UV to valid range first, then convert to pixel coordinates
		let u_clamped = u.clamp(0.0, 1.0);
		let v_clamped = v.clamp(0.0, 1.0);

		// Use (width - 1) as max to map [0,1] to [0, width-1]
		let x = (u_clamped * (self.width - 1) as f32).round() as u32;
		let y = (v_clamped * (self.height - 1) as f32).round() as u32;

		self.image
			.get_pixel(x.min(self.width - 1), y.min(self.height - 1))
	}

	/// Sample a pixel using bilinear filtering for smoother, softer appearance
	///
	/// Interpolates between 4 neighboring pixels. Uses alpha-aware filtering
	/// to avoid transparency artifacts at boundaries.
	pub fn sample_uv_bilinear(&self, u: f32, v: f32) -> Rgba<u8> {
		let x = u * self.width as f32;
		let y = v * self.height as f32;

		let x0 = x.floor().clamp(0.0, (self.width - 1) as f32) as u32;
		let y0 = y.floor().clamp(0.0, (self.height - 1) as f32) as u32;
		let x1 = (x0 + 1).min(self.width - 1);
		let y1 = (y0 + 1).min(self.height - 1);

		let fx = x - x0 as f32;
		let fy = y - y0 as f32;

		let p00 = self.image.get_pixel(x0, y0);
		let p10 = self.image.get_pixel(x1, y0);
		let p01 = self.image.get_pixel(x0, y1);
		let p11 = self.image.get_pixel(x1, y1);

		// Alpha discontinuity handling: if mixing opaque and transparent, fallback to nearest
		let alpha_threshold = 128;
		let alphas = [p00[3], p10[3], p01[3], p11[3]];
		let has_opaque = alphas.iter().any(|&a| a >= alpha_threshold);
		let has_transparent = alphas.iter().any(|&a| a < alpha_threshold);

		if has_opaque && has_transparent {
			let nearest_x = if fx < 0.5 { x0 } else { x1 };
			let nearest_y = if fy < 0.5 { y0 } else { y1 };
			return self.image.get_pixel(nearest_x, nearest_y);
		}

		let lerp = |a: u8, b: u8, t: f32| -> u8 { ((a as f32 * (1.0 - t)) + (b as f32 * t)) as u8 };

		let lerp_rgba = |a: Rgba<u8>, b: Rgba<u8>, t: f32| -> Rgba<u8> {
			Rgba([
				lerp(a[0], b[0], t),
				lerp(a[1], b[1], t),
				lerp(a[2], b[2], t),
				lerp(a[3], b[3], t),
			])
		};

		let top = lerp_rgba(p00, p10, fx);
		let bottom = lerp_rgba(p01, p11, fx);

		lerp_rgba(top, bottom, fy)
	}

	pub fn get_pixel(&self, x: u32, y: u32) -> Rgba<u8> {
		let x = x.min(self.width - 1);
		let y = y.min(self.height - 1);
		self.image.get_pixel(x, y)
	}

	/// Create a texture from an image (for testing and internal use)
	pub fn from_image(image: DynamicImage) -> Self {
		let (width, height) = image.dimensions();
		Texture {
			image,
			width,
			height,
		}
	}
}

/// A 1D tint gradient for colorizing greyscale textures
///
/// The greyscale value from the texture is used as an X-coordinate lookup.
/// For fabric materials, the lookup can be inverted.
#[derive(Debug, Clone)]
pub struct TintGradient {
	pixels: Vec<Rgba<u8>>,
	inverted: bool,
	brightness: f32,
}

impl TintGradient {
	pub fn from_file(path: &Path) -> Result<Self> {
		let image = image::open(path)?;
		Ok(Self::from_image(&image))
	}

	pub fn from_bytes(data: &[u8]) -> Result<Self> {
		let image = image::load_from_memory(data)?;
		Ok(Self::from_image(&image))
	}

	pub fn from_image(image: &DynamicImage) -> Self {
		let (width, height) = image.dimensions();
		let mut pixels = Vec::with_capacity(width as usize);

		let y = height / 2;
		for x in 0..width {
			pixels.push(image.get_pixel(x, y));
		}

		TintGradient {
			pixels,
			inverted: false,
			brightness: 1.0,
		}
	}

	pub fn solid(color: Rgba<u8>) -> Self {
		TintGradient {
			pixels: vec![color; 256],
			inverted: false,
			brightness: 1.0,
		}
	}

	pub fn identity() -> Self {
		let pixels: Vec<Rgba<u8>> = (0..=255)
			.map(|i| Rgba([i as u8, i as u8, i as u8, 255]))
			.collect();
		TintGradient {
			pixels,
			inverted: false,
			brightness: 1.0,
		}
	}

	/// Create a gradient from a list of base colors
	///
	/// - 1 color: Solid tint
	/// - 2+ colors: Linear interpolation between points
	pub fn from_base_colors(colors: &[Rgba<u8>]) -> Self {
		if colors.is_empty() {
			return Self::identity();
		}
		if colors.len() == 1 {
			return Self::solid(colors[0]);
		}

		let mut pixels = Vec::with_capacity(256);
		let num_segments = colors.len() - 1;

		for i in 0..256 {
			let t_global = i as f32 / 255.0; // 0.0 to 1.0

			// Map global t to segment
			let segment_t = t_global * num_segments as f32;
			let index = segment_t.floor() as usize;
			let index = index.min(num_segments - 1);
			let t = segment_t - index as f32;

			let c1 = colors[index];
			let c2 = colors[index + 1];

			let r = (c1[0] as f32 * (1.0 - t) + c2[0] as f32 * t) as u8;
			let g = (c1[1] as f32 * (1.0 - t) + c2[1] as f32 * t) as u8;
			let b = (c1[2] as f32 * (1.0 - t) + c2[2] as f32 * t) as u8;
			let a = 255;

			pixels.push(Rgba([r, g, b, a]));
		}

		TintGradient {
			pixels,
			inverted: false,
			brightness: 1.0,
		}
	}

	pub fn from_hex_colors(hex_colors: &[String]) -> Self {
		let colors: Vec<Rgba<u8>> = hex_colors
			.iter()
			.filter_map(|hex| {
				let s = hex.trim_start_matches('#');
				if s.len() == 6 {
					let r = u8::from_str_radix(&s[0..2], 16).ok()?;
					let g = u8::from_str_radix(&s[2..4], 16).ok()?;
					let b = u8::from_str_radix(&s[4..6], 16).ok()?;
					Some(Rgba([r, g, b, 255]))
				} else {
					None
				}
			})
			.collect();

		Self::from_base_colors(&colors)
	}

	pub fn with_inverted(mut self, inverted: bool) -> Self {
		self.inverted = inverted;
		self
	}

	pub fn with_brightness(mut self, brightness: f32) -> Self {
		self.brightness = brightness;
		self
	}

	/// Lookup a color by greyscale value (0.0 to 1.0)
	pub fn lookup(&self, grey: f32) -> Rgba<u8> {
		if self.pixels.is_empty() {
			return Rgba([255, 255, 255, 255]);
		}

		let len = self.pixels.len() as f32;
		let index = (grey * (len - 1.0) + 0.5).clamp(0.0, len - 1.0) as usize;
		self.pixels[index.min(self.pixels.len() - 1)]
	}

	/// Lookup by integer greyscale value (0-255)
	pub fn lookup_u8(&self, grey: u8) -> Rgba<u8> {
		let mut effective_grey = if self.inverted { 255 - grey } else { grey };

		if self.brightness != 1.0 {
			effective_grey = ((effective_grey as f32 * self.brightness)
				.round()
				.clamp(0.0, 255.0)) as u8;
		}

		self.lookup(effective_grey as f32 / 255.0)
	}

	pub fn len(&self) -> usize {
		self.pixels.len()
	}

	pub fn is_empty(&self) -> bool {
		self.pixels.is_empty()
	}
}

/// Apply a tint gradient to a greyscale pixel
///
/// This function only tints pixels that are greyscale (where R ≈ G ≈ B).
/// Pre-colored pixels (where R, G, B differ significantly) are preserved.
/// This allows textures to have both tintable greyscale areas and fixed-color decorative elements.
pub fn apply_tint(pixel: Rgba<u8>, gradient: &TintGradient) -> Rgba<u8> {
	// Early exit for transparent pixels
	if pixel[3] == 0 {
		return pixel;
	}

	// Detect greyscale: check if R ≈ G ≈ B
	let min = pixel[0].min(pixel[1]).min(pixel[2]);
	let max = pixel[0].max(pixel[1]).max(pixel[2]);
	let deviation = max - min;

	if deviation <= 1 {
		// Greyscale threshold
		// Tint greyscale pixels using average luminance
		let luminance = ((pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3) as u8;
		let mut tinted = gradient.lookup_u8(luminance);
		tinted[3] = pixel[3]; // Preserve alpha
		tinted
	} else {
		// Preserve colored pixels
		pixel
	}
}

/// Transform UV coordinates based on face settings
pub fn transform_uv_coords(face: &UvFace, size_x: f32, size_y: f32, u: f32, v: f32) -> (f32, f32) {
	// Start with offset
	let mut tex_u = face.offset.x + u * size_x;
	let mut tex_v = face.offset.y + v * size_y;

	// Apply mirror
	if face.mirror.x {
		tex_u = face.offset.x - (u * size_x);
	}
	if face.mirror.y {
		tex_v = face.offset.y - (v * size_y);
	}

	// Apply rotation
	let (rotated_u, rotated_v) =
		apply_rotation(tex_u, tex_v, face.offset, size_x, size_y, face.angle);

	(rotated_u, rotated_v)
}

fn apply_rotation(
	u: f32,
	v: f32,
	offset: UvOffset,
	_size_x: f32,
	_size_y: f32,
	angle: UvAngle,
) -> (f32, f32) {
	// Calculate relative position from offset (before rotation was applied)
	let rel_u = u - offset.x;
	let rel_v = v - offset.y;

	// The rotation describes how the texture region was rotated when authored.
	// We need to reverse the rotation to find the correct texture coordinates.
	//
	// Based on Blockbench plugin behavior:
	// - Angle 0: offset is top-left, region extends Right (+X) and Down (+Y)
	// - Angle 90: offset is top-right, region extends Down (+Y) and Left (-X), axes swapped
	// - Angle 180: offset is bottom-right, region extends Left (-X) and Up (-Y)
	// - Angle 270: offset is bottom-left, region extends Up (-Y) and Right (+X), axes swapped
	//
	// For rotated UVs (90/270), the size dimensions are effectively swapped in the texture.
	match angle.as_degrees() {
		0 => {
			// No rotation: offset is top-left
			(u, v)
		}
		90 => {
			// Rotated 90 degrees CW: offset is top-right of texture region
			// U maps to -V direction, V maps to +U direction in texture
			// Texture region extends: Left (-X) by size_y, Down (+Y) by size_x
			let new_u = offset.x - rel_v;
			let new_v = offset.y + rel_u;
			(new_u, new_v)
		}
		180 => {
			// Rotated 180 degrees: offset is bottom-right of texture region
			// Both axes are inverted
			let new_u = offset.x - rel_u;
			let new_v = offset.y - rel_v;
			(new_u, new_v)
		}
		270 => {
			// Rotated 270 degrees CW (= 90 CCW): offset is bottom-left of texture region
			// U maps to +V direction, V maps to -U direction in texture
			// Texture region extends: Right (+X) by size_y, Up (-Y) by size_x
			let new_u = offset.x + rel_v;
			let new_v = offset.y - rel_u; // Correct: V goes up (decreasing)
			(new_u, new_v)
		}
		_ => (u, v), // Unknown angle, return unchanged
	}
}

/// Sample texture for a face using UV coordinates
pub fn sample_face_texture(
	texture: &Texture,
	face: &UvFace,
	size_x: f32,
	size_y: f32,
	u: f32,
	v: f32,
) -> Rgba<u8> {
	let (tex_u, tex_v) = transform_uv_coords(face, size_x, size_y, u, v);

	// Use direct pixel sampling to avoid precision loss from UV normalization
	texture.sample_pixel(tex_u, tex_v)
}

/// Sample texture for a face using bilinear filtering for smoother appearance
pub fn sample_face_texture_bilinear(
	texture: &Texture,
	face: &UvFace,
	size_x: f32,
	size_y: f32,
	u: f32,
	v: f32,
) -> Rgba<u8> {
	// Apply small epsilon inset to avoid sampling at exact boundaries
	let epsilon = 0.001;
	let u_safe = u.clamp(epsilon, 1.0 - epsilon);
	let v_safe = v.clamp(epsilon, 1.0 - epsilon);

	let (tex_u, tex_v) = transform_uv_coords(face, size_x, size_y, u_safe, v_safe);

	// Convert to 0-1 UV coordinates
	let (width, height) = texture.dimensions();
	let uv_u = tex_u / width as f32;
	let uv_v = tex_v / height as f32;

	texture.sample_uv_bilinear(uv_u, uv_v)
}

/// Sample texture for a face with tint gradient applied
///
/// This samples the greyscale texture and uses the luminance value
/// to look up the final color from the tint gradient.
pub fn sample_face_texture_tinted(
	texture: &Texture,
	face: &UvFace,
	size_x: f32,
	size_y: f32,
	u: f32,
	v: f32,
	tint: &TintGradient,
) -> Rgba<u8> {
	let pixel = sample_face_texture(texture, face, size_x, size_y, u, v);
	apply_tint(pixel, tint)
}

/// Sample texture for a face with tint gradient applied using bilinear filtering
pub fn sample_face_texture_tinted_bilinear(
	texture: &Texture,
	face: &UvFace,
	size_x: f32,
	size_y: f32,
	u: f32,
	v: f32,
	tint: &TintGradient,
) -> Rgba<u8> {
	let pixel = sample_face_texture_bilinear(texture, face, size_x, size_y, u, v);
	apply_tint(pixel, tint)
}

/// Map Hytale face name to texture face
pub fn get_texture_face(face_name: &str) -> Option<&str> {
	match face_name {
		"front" | "back" | "left" | "right" | "top" | "bottom" => Some(face_name),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::{UvAngle, UvFace, UvMirror, UvOffset};
	use image::{Rgba, RgbaImage};

	fn create_test_texture() -> Texture {
		// Create a simple 64x64 test texture
		let mut img = RgbaImage::new(64, 64);
		// Fill with a pattern
		for y in 0..64 {
			for x in 0..64 {
				let pixel = Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255]);
				img.put_pixel(x, y, pixel);
			}
		}
		Texture {
			image: image::DynamicImage::ImageRgba8(img),
			width: 64,
			height: 64,
		}
	}

	#[test]
	fn test_load_texture_from_bytes() {
		// Create a minimal PNG in memory
		let mut png_data = Vec::new();
		let img = RgbaImage::new(16, 16);
		let mut cursor = std::io::Cursor::new(&mut png_data);
		image::DynamicImage::ImageRgba8(img)
			.write_to(&mut cursor, image::ImageFormat::Png)
			.unwrap();

		let texture = Texture::from_bytes(&png_data).unwrap();
		assert_eq!(texture.dimensions(), (16, 16));
	}

	#[test]
	fn test_handle_missing_texture_file() {
		let path = std::path::Path::new("nonexistent_texture.png");
		let result = Texture::from_file(path);
		assert!(result.is_err());
	}

	#[test]
	fn test_uv_coordinate_mapping_offset() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 20.0 },
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(0),
		};

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 20.0).abs() < 0.001);

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 1.0);
		assert!((u - 18.0).abs() < 0.001); // 10 + 8
		assert!((v - 32.0).abs() < 0.001); // 20 + 12
	}

	#[test]
	fn test_mirror_x_axis() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 20.0 },
			mirror: UvMirror { x: true, y: false },
			angle: UvAngle(0),
		};

		let (u, _v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		// u=0 should be offset.x = 10
		assert!((u - 10.0).abs() < 0.001);

		let (u, _v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 0.0);
		// u=1 should be offset.x - size_x = 10 - 8 = 2
		assert!((u - 2.0).abs() < 0.001);
	}

	#[test]
	fn test_mirror_y_axis() {
		let face = UvFace {
			offset: UvOffset { x: 0.0, y: 0.0 },
			mirror: UvMirror { x: false, y: true },
			angle: UvAngle(0),
		};

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		// v=0 should be offset.y = 0
		assert!((u - 0.0).abs() < 0.001);
		assert!((v - 0.0).abs() < 0.001);

		let (_u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 1.0);
		// v=1 should be offset.y - size_y = 0 - 12 = -12
		assert!((v - (-12.0)).abs() < 0.001);
	}

	#[test]
	fn test_rotation_0_degrees() {
		let face = UvFace {
			offset: UvOffset { x: 0.0, y: 0.0 },
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(0),
		};

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.5, 0.5);
		assert!((u - 4.0).abs() < 0.001);
		assert!((v - 6.0).abs() < 0.001);
	}

	#[test]
	fn test_rotation_90_degrees() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 10.0 }, // Anchor at 10,10
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(90),
		};

		// Size 8x12
		// Face TL (0,0) -> Tex (ox, oy) = (10, 10)
		// Face TR (1,0) -> rel_u=8, rel_v=0 -> new_u=10-0=10, new_v=10+8=18
		// Face BL (0,1) -> rel_u=0, rel_v=12 -> new_u=10-12=-2, new_v=10+0=10

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 10.0).abs() < 0.001);

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 0.0); // rel_u=8, rel_v=0
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 18.0).abs() < 0.001);

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 1.0); // rel_u=0, rel_v=12
		assert!((u - (-2.0)).abs() < 0.001);
		assert!((v - 10.0).abs() < 0.001);
	}

	#[test]
	fn test_rotation_180_degrees() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 10.0 },
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(180),
		};

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 10.0).abs() < 0.001);

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 1.0); // rel_u=8, rel_v=12
		assert!((u - 2.0).abs() < 0.001);
		assert!((v - (-2.0)).abs() < 0.001);
	}

	#[test]
	fn test_rotation_270_degrees() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 10.0 },
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(270),
		};

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 10.0).abs() < 0.001);

		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 1.0); // rel_u=8, rel_v=12
		assert!((u - 22.0).abs() < 0.001);
		assert!((v - 2.0).abs() < 0.001);
	}

	#[test]
	fn test_combined_mirror_and_rotation() {
		let face = UvFace {
			offset: UvOffset { x: 10.0, y: 20.0 },
			mirror: UvMirror { x: true, y: true },
			angle: UvAngle(90),
		};

		// Size 8x12
		// u=0,v=0 -> mirror -> tex_u=10, tex_v=20 -> rot90 -> u=10, v=20
		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 0.0, 0.0);
		assert!((u - 10.0).abs() < 0.001);
		assert!((v - 20.0).abs() < 0.001);

		// u=1,v=1 -> mirror -> tex_u=10-8=2, tex_v=20-12=8 -> rel_u=-8, rel_v=-12
		// rot90 -> new_u = 10 - (-12) = 22, new_v = 20 + (-8) = 12
		let (u, v) = transform_uv_coords(&face, 8.0, 12.0, 1.0, 1.0);
		assert!((u - 22.0).abs() < 0.001);
		assert!((v - 12.0).abs() < 0.001);
	}

	#[test]
	fn test_face_name_mapping() {
		assert_eq!(get_texture_face("front"), Some("front"));
		assert_eq!(get_texture_face("back"), Some("back"));
		assert_eq!(get_texture_face("left"), Some("left"));
		assert_eq!(get_texture_face("right"), Some("right"));
		assert_eq!(get_texture_face("top"), Some("top"));
		assert_eq!(get_texture_face("bottom"), Some("bottom"));
		assert_eq!(get_texture_face("invalid"), None);
	}

	#[test]
	fn test_sample_texture_pixels() {
		let texture = create_test_texture();

		// Sample at center
		let pixel = texture.sample_uv(0.5, 0.5);
		assert_eq!(pixel[3], 255); // Should be opaque

		// Sample at corner
		let pixel = texture.sample_uv(0.0, 0.0);
		assert_eq!(pixel[3], 255);

		// Sample at other corner
		let pixel = texture.sample_uv(1.0, 1.0);
		assert_eq!(pixel[3], 255);
	}

	#[test]
	fn test_handle_out_of_bounds_uv_coordinates() {
		let texture = create_test_texture();

		// UV coordinates outside 0-1 range should be clamped
		let pixel1 = texture.sample_uv(-0.1, -0.1);
		let pixel2 = texture.sample_uv(0.0, 0.0);
		assert_eq!(pixel1, pixel2);

		let pixel1 = texture.sample_uv(1.5, 1.5);
		let pixel2 = texture.sample_uv(1.0, 1.0);
		assert_eq!(pixel1, pixel2);
	}

	#[test]
	fn test_sample_face_texture() {
		let texture = create_test_texture();
		let face = UvFace {
			offset: UvOffset { x: 0.0, y: 0.0 },
			mirror: UvMirror { x: false, y: false },
			angle: UvAngle(0),
		};

		let pixel = sample_face_texture(&texture, &face, 8.0, 12.0, 0.0, 0.0);
		assert_eq!(pixel[3], 255); // Should be valid pixel
	}

	// TintGradient tests

	fn create_test_gradient() -> TintGradient {
		// Create a gradient from black to white
		let mut img = RgbaImage::new(256, 1);
		for x in 0..256 {
			img.put_pixel(x, 0, Rgba([x as u8, x as u8, x as u8, 255]));
		}
		TintGradient::from_image(&image::DynamicImage::ImageRgba8(img))
	}

	fn create_colored_gradient() -> TintGradient {
		// Create a gradient from dark brown to light peach (like skin tone)
		let mut img = RgbaImage::new(256, 1);
		for x in 0..256 {
			let t = x as f32 / 255.0;
			let r = (80.0 + t * 175.0) as u8; // 80 -> 255
			let g = (40.0 + t * 180.0) as u8; // 40 -> 220
			let b = (30.0 + t * 170.0) as u8; // 30 -> 200
			img.put_pixel(x, 0, Rgba([r, g, b, 255]));
		}
		TintGradient::from_image(&image::DynamicImage::ImageRgba8(img))
	}

	#[test]
	fn test_tint_gradient_identity() {
		let gradient = TintGradient::identity();
		assert_eq!(gradient.len(), 256);

		// Black should return black
		let black = gradient.lookup(0.0);
		assert_eq!(black[0], 0);
		assert_eq!(black[1], 0);
		assert_eq!(black[2], 0);

		// White should return white
		let white = gradient.lookup(1.0);
		assert_eq!(white[0], 255);
		assert_eq!(white[1], 255);
		assert_eq!(white[2], 255);

		// Mid-grey should return mid-grey
		let mid = gradient.lookup(0.5);
		assert!((mid[0] as i32 - 127).abs() <= 1);
	}

	#[test]
	fn test_tint_gradient_solid() {
		let red = Rgba([255, 0, 0, 255]);
		let gradient = TintGradient::solid(red);

		// All lookups should return the same color
		assert_eq!(gradient.lookup(0.0), red);
		assert_eq!(gradient.lookup(0.5), red);
		assert_eq!(gradient.lookup(1.0), red);
	}

	#[test]
	fn test_tint_gradient_lookup() {
		let gradient = create_test_gradient();

		// Dark end
		let dark = gradient.lookup(0.0);
		assert!(dark[0] < 10);

		// Light end
		let light = gradient.lookup(1.0);
		assert!(light[0] > 245);

		// Middle
		let mid = gradient.lookup(0.5);
		assert!(mid[0] > 100 && mid[0] < 160);
	}

	#[test]
	fn test_tint_gradient_lookup_u8() {
		let gradient = create_test_gradient();

		let dark = gradient.lookup_u8(0);
		let light = gradient.lookup_u8(255);

		assert!(dark[0] < 10);
		assert!(light[0] > 245);
	}

	#[test]
	fn test_apply_tint() {
		let gradient = create_colored_gradient();

		// Dark pixel should map to dark end of gradient
		let dark_pixel = Rgba([50, 50, 50, 255]);
		let tinted_dark = apply_tint(dark_pixel, &gradient);
		// Should be brownish (more red than green/blue)
		assert!(tinted_dark[0] > tinted_dark[2]);

		// Light pixel should map to light end of gradient
		let light_pixel = Rgba([200, 200, 200, 255]);
		let tinted_light = apply_tint(light_pixel, &gradient);
		// Should be lighter/peachy
		assert!(tinted_light[0] > 150);
		assert!(tinted_light[1] > 100);
	}

	#[test]
	fn test_apply_tint_preserves_alpha() {
		let gradient = TintGradient::identity();

		// Transparent pixel
		let transparent = Rgba([128, 128, 128, 64]);
		let tinted = apply_tint(transparent, &gradient);
		assert_eq!(tinted[3], 64); // Alpha preserved

		// Opaque pixel
		let opaque = Rgba([128, 128, 128, 255]);
		let tinted = apply_tint(opaque, &gradient);
		assert_eq!(tinted[3], 255); // Alpha preserved
	}

	#[test]
	fn test_apply_tint_greyscale_pixels() {
		let gradient = create_colored_gradient();

		// Pure greyscale pixel (R=G=B) should be tinted
		let grey_pixel = Rgba([128, 128, 128, 255]);
		let tinted = apply_tint(grey_pixel, &gradient);

		// Should be tinted (not equal to original)
		assert_ne!(tinted[0], 128);
		assert_ne!(tinted, grey_pixel);

		// Alpha should be preserved
		assert_eq!(tinted[3], 255);
	}

	#[test]
	fn test_apply_tint_colored_pixels_preserved() {
		let gradient = create_colored_gradient();

		// Brown decorative element (clearly colored, deviation > 8)
		let brown_pixel = Rgba([139, 69, 19, 255]); // Saddle brown
		let tinted = apply_tint(brown_pixel, &gradient);

		// Should NOT be tinted - pixel should be identical
		assert_eq!(tinted, brown_pixel);
		assert_eq!(tinted[0], 139);
		assert_eq!(tinted[1], 69);
		assert_eq!(tinted[2], 19);
		assert_eq!(tinted[3], 255);
	}

	#[test]
	fn test_apply_tint_threshold_boundary() {
		let gradient = create_colored_gradient();

		// Pixel at threshold (deviation = 8) should be tinted
		let at_threshold = Rgba([128, 128, 136, 255]); // deviation = 8
		let tinted_at = apply_tint(at_threshold, &gradient);
		assert_ne!(tinted_at, at_threshold); // Should be tinted

		// Pixel just over threshold (deviation = 9) should be preserved
		let over_threshold = Rgba([128, 128, 137, 255]); // deviation = 9
		let tinted_over = apply_tint(over_threshold, &gradient);
		assert_eq!(tinted_over, over_threshold); // Should NOT be tinted
	}

	#[test]
	fn test_apply_tint_transparent_pixels() {
		let gradient = create_colored_gradient();

		// Fully transparent pixel should be returned unchanged (early exit)
		let transparent = Rgba([100, 50, 25, 0]);
		let tinted = apply_tint(transparent, &gradient);
		assert_eq!(tinted, transparent);

		// Even if it would otherwise be colored
		let transparent_colored = Rgba([255, 0, 0, 0]);
		let tinted_colored = apply_tint(transparent_colored, &gradient);
		assert_eq!(tinted_colored, transparent_colored);
	}

	#[test]
	fn test_apply_tint_near_greyscale() {
		let gradient = create_colored_gradient();

		// Near-greyscale pixels (slight RGB variation due to compression artifacts)
		// deviation <= 8 should still be tinted
		let near_grey_1 = Rgba([100, 102, 100, 255]); // deviation = 2
		let tinted_1 = apply_tint(near_grey_1, &gradient);
		assert_ne!(tinted_1, near_grey_1); // Should be tinted

		let near_grey_2 = Rgba([150, 145, 150, 255]); // deviation = 5
		let tinted_2 = apply_tint(near_grey_2, &gradient);
		assert_ne!(tinted_2, near_grey_2); // Should be tinted
	}

	#[test]
	fn test_apply_tint_uses_average_luminance() {
		let gradient = create_colored_gradient();

		// Test that average luminance is used, not just red channel
		// RGB(90, 120, 150) has average 120
		let _pixel = Rgba([90, 120, 150, 255]); // deviation = 60, won't be tinted

		// But for a greyscale version with same average
		let greyscale = Rgba([120, 120, 120, 255]); // average = 120
		let tinted = apply_tint(greyscale, &gradient);

		// Should use luminance value of 120 for gradient lookup
		// Verify it was tinted (not equal to original)
		assert_ne!(tinted, greyscale);
		assert_eq!(tinted[3], 255); // Alpha preserved
	}

	#[test]
	fn test_tint_gradient_clamps_out_of_range() {
		let gradient = create_test_gradient();

		// Values outside 0-1 should be clamped
		let below = gradient.lookup(-0.5);
		let at_zero = gradient.lookup(0.0);
		assert_eq!(below, at_zero);

		let above = gradient.lookup(1.5);
		let at_one = gradient.lookup(1.0);
		assert_eq!(above, at_one);
	}

	#[test]
	fn test_tint_gradient_inverted() {
		let gradient = create_test_gradient();
		let inverted_gradient = create_test_gradient().with_inverted(true);

		// Normal: dark grey (0) → dark color (left of gradient)
		// Inverted: dark grey (0) → bright color (right of gradient)
		let dark_normal = gradient.lookup_u8(0);
		let dark_inverted = inverted_gradient.lookup_u8(0);

		// Normal: bright grey (255) → bright color (right of gradient)
		// Inverted: bright grey (255) → dark color (left of gradient)
		let bright_normal = gradient.lookup_u8(255);
		let bright_inverted = inverted_gradient.lookup_u8(255);

		// Inverted dark should equal normal bright
		assert_eq!(dark_inverted, bright_normal);
		// Inverted bright should equal normal dark
		assert_eq!(bright_inverted, dark_normal);
	}
}
