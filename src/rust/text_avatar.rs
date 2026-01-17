//! Text-based avatar rendering for Hytale
//!
//! TEMPORARY: This module renders username initials as avatars until real Hytale
//! skin support is implemented. Once Hytale skin textures are available, this
//! can be replaced with proper skin-based rendering like Minecraft.

use image::{Rgba, RgbaImage};

/// Simple 5x7 pixel font for uppercase letters and digits
/// Each character is represented as 7 rows of 5 bits (stored as u8)
#[rustfmt::skip]
const FONT_5X7: [([u8; 7], char); 36] = [
    ([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001], 'A'),
    ([0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110], 'B'),
    ([0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110], 'C'),
    ([0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110], 'D'),
    ([0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b11111], 'E'),
    ([0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000], 'F'),
    ([0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110], 'G'),
    ([0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0b10001], 'H'),
    ([0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], 'I'),
    ([0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100], 'J'),
    ([0b10001, 0b10010, 0b11100, 0b10010, 0b10001, 0b10001, 0b10001], 'K'),
    ([0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111], 'L'),
    ([0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001], 'M'),
    ([0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001], 'N'),
    ([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110], 'O'),
    ([0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000], 'P'),
    ([0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101], 'Q'),
    ([0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001], 'R'),
    ([0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110], 'S'),
    ([0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100], 'T'),
    ([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110], 'U'),
    ([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100], 'V'),
    ([0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010], 'W'),
    ([0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001], 'X'),
    ([0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100], 'Y'),
    ([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111], 'Z'),
    ([0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110], '0'),
    ([0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], '1'),
    ([0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111], '2'),
    ([0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110], '3'),
    ([0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010], '4'),
    ([0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110], '5'),
    ([0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110], '6'),
    ([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000], '7'),
    ([0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110], '8'),
    ([0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100], '9'),
];

/// Get the font data for a character
fn get_char_data(c: char) -> Option<[u8; 7]> {
	let upper = c.to_ascii_uppercase();
	FONT_5X7
		.iter()
		.find(|(_, ch)| *ch == upper)
		.map(|(data, _)| *data)
}

/// Simple hash function for generating deterministic colors
fn hash_username(username: &str) -> u32 {
	let mut hash: u32 = 0;
	for byte in username.to_lowercase().bytes() {
		hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
	}
	hash
}

/// Calculate relative luminance of an RGB color (0.0 to 1.0)
/// Uses sRGB luminance coefficients per WCAG guidelines
fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
	let r = r as f32 / 255.0;
	let g = g as f32 / 255.0;
	let b = b as f32 / 255.0;
	0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Choose contrasting text color (white or dark) based on background luminance
fn contrasting_text_color(bg: Rgba<u8>) -> Rgba<u8> {
	let luminance = relative_luminance(bg[0], bg[1], bg[2]);
	// Use white text on dark backgrounds, dark text on light backgrounds
	// Threshold of 0.5 provides good contrast in both cases
	if luminance > 0.5 {
		Rgba([30, 30, 30, 255]) // Dark gray for light backgrounds
	} else {
		Rgba([255, 255, 255, 255]) // White for dark backgrounds
	}
}

/// Generate a pleasing background color from username hash
/// Uses HSL-like approach to get saturated colors
fn username_to_color(username: &str) -> Rgba<u8> {
	let hash = hash_username(username);

	// Use hash to determine hue (0-360 degrees mapped to color)
	let hue: f32 = (hash % 360) as f32;
	// Fixed saturation and lightness for pleasing colors
	let saturation: f32 = 0.65;
	let lightness: f32 = 0.45;

	// HSL to RGB conversion
	let c: f32 = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
	let x: f32 = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
	let m: f32 = lightness - c / 2.0;

	let (r, g, b): (f32, f32, f32) = match (hue / 60.0) as u32 {
		0 => (c, x, 0.0),
		1 => (x, c, 0.0),
		2 => (0.0, c, x),
		3 => (0.0, x, c),
		4 => (x, 0.0, c),
		_ => (c, 0.0, x),
	};

	Rgba([
		((r + m) * 255.0) as u8,
		((g + m) * 255.0) as u8,
		((b + m) * 255.0) as u8,
		255,
	])
}

/// Extract initials from username
/// "CherryJimbo" -> "CJ", "james" -> "J", "AB" -> "AB"
fn extract_initials(username: &str) -> String {
	let chars: Vec<char> = username.chars().collect();

	if chars.is_empty() {
		return "?".to_string();
	}

	// For short usernames (2 chars or less), just return them uppercased
	if chars.len() <= 2 {
		return chars.iter().map(|c| c.to_ascii_uppercase()).collect();
	}

	// Find capital letters for CamelCase detection
	let mut initials = String::new();
	let mut prev_was_lower = false;

	for (i, c) in chars.iter().enumerate() {
		if i == 0 {
			// Always include first character
			initials.push(c.to_ascii_uppercase());
			prev_was_lower = c.is_ascii_lowercase();
		} else if c.is_ascii_uppercase() && prev_was_lower && initials.len() < 2 {
			// CamelCase transition
			initials.push(*c);
		} else {
			prev_was_lower = c.is_ascii_lowercase();
		}

		if initials.len() >= 2 {
			break;
		}
	}

	initials
}

/// Draw a single character at the specified position with scaling
fn draw_char(image: &mut RgbaImage, c: char, x: i32, y: i32, scale: u32, color: Rgba<u8>) {
	if let Some(char_data) = get_char_data(c) {
		for (row_idx, row) in char_data.iter().enumerate() {
			for col in 0..5 {
				if (row >> (4 - col)) & 1 == 1 {
					// Draw scaled pixel
					for sy in 0..scale {
						for sx in 0..scale {
							let px = x + (col * scale as i32) + sx as i32;
							let py = y + (row_idx as u32 * scale) as i32 + sy as i32;
							if px >= 0
								&& py >= 0 && (px as u32) < image.width()
								&& (py as u32) < image.height()
							{
								image.put_pixel(px as u32, py as u32, color);
							}
						}
					}
				}
			}
		}
	}
}

/// Render a text avatar with username initials
pub fn render_text_avatar(username: &str, size: u32) -> RgbaImage {
	let bg_color = username_to_color(username);
	let text_color = contrasting_text_color(bg_color);

	let initials = extract_initials(username);
	let num_chars = initials.len();

	// Create image with background color
	let mut image = RgbaImage::from_pixel(size, size, bg_color);

	// Calculate scale based on size
	// Font is 5x7, we want it to be about 60% of the image height
	let target_height = (size as f32 * 0.6) as u32;
	let scale = (target_height / 7).max(1);

	let char_width = 5 * scale;
	let char_height = 7 * scale;
	let spacing = scale; // Space between characters

	// Calculate total text width
	let total_width = if num_chars == 1 {
		char_width
	} else {
		char_width * 2 + spacing
	};

	// Center the text
	let start_x = ((size - total_width) / 2) as i32;
	let start_y = ((size - char_height) / 2) as i32;

	// Draw each character
	for (i, c) in initials.chars().enumerate() {
		let x = start_x + (i as i32 * (char_width as i32 + spacing as i32));
		draw_char(&mut image, c, x, start_y, scale, text_color);
	}

	image
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_initials_camelcase() {
		assert_eq!(extract_initials("CherryJimbo"), "CJ");
	}

	#[test]
	fn test_extract_initials_single() {
		assert_eq!(extract_initials("james"), "J");
	}

	#[test]
	fn test_extract_initials_short() {
		assert_eq!(extract_initials("AB"), "AB");
	}

	#[test]
	fn test_extract_initials_empty() {
		assert_eq!(extract_initials(""), "?");
	}

	#[test]
	fn test_username_color_deterministic() {
		let color1 = username_to_color("test");
		let color2 = username_to_color("test");
		assert_eq!(color1, color2);
	}

	#[test]
	fn test_username_color_different() {
		let color1 = username_to_color("alice");
		let color2 = username_to_color("bob");
		assert_ne!(color1, color2);
	}

	#[test]
	fn test_contrasting_text_dark_bg() {
		// Dark background should get white text
		let dark_bg = Rgba([30, 30, 30, 255]);
		let text = contrasting_text_color(dark_bg);
		assert_eq!(text, Rgba([255, 255, 255, 255]));
	}

	#[test]
	fn test_contrasting_text_light_bg() {
		// Light background should get dark text
		let light_bg = Rgba([200, 200, 200, 255]);
		let text = contrasting_text_color(light_bg);
		assert_eq!(text, Rgba([30, 30, 30, 255]));
	}
}
