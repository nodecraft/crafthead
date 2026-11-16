//! Core triangle rasterization with depth buffer and texture sampling
//!
//! Provides per-pixel triangle rendering with:
//! - Depth testing via z-buffer
//! - UV interpolation using barycentric coordinates
//! - Texture sampling with optional bilinear filtering
//! - Tinting and gamma-correct diffuse shading

use crate::models::UvFace;
use crate::texture::{
	apply_tint, sample_face_texture, sample_face_texture_bilinear, Texture, TintGradient,
};
use image::{Rgba, RgbaImage};
use std::collections::HashMap;
use std::sync::OnceLock;

use super::math::Barycentric;

/// Small depth bias to prevent Z-fighting between coplanar surfaces.
/// A surface must be this much closer than the current depth to overwrite it.
const DEPTH_BIAS: f32 = 0.001;

const GAMMA: f32 = 2.2;

/// Marks a shade table entry that has not been computed yet.
const UNSHADED: u16 = u16::MAX;

/// sRGB channel values converted to linear light; identical for every render.
fn linear_channels() -> &'static [f32; 256] {
	static LINEAR: OnceLock<[f32; 256]> = OnceLock::new();
	LINEAR.get_or_init(|| std::array::from_fn(|c| (c as f32 / 255.0).powf(GAMMA)))
}

/// Gamma-correct shading results, memoised per light level.
///
/// Lighting is constant across a face, so the shaded value of a channel depends only on
/// (light level, channel value). Caching it replaces two `powf` calls per channel per pixel.
#[derive(Default)]
pub(crate) struct ShadeCache {
	level_index: HashMap<u32, usize>,
	levels: Vec<ShadeLevel>,
}

pub(crate) struct ShadeLevel {
	lighting: f32,
	shaded: [u16; 256],
}

impl ShadeCache {
	pub(crate) fn level(&mut self, lighting: f32) -> &mut ShadeLevel {
		let levels = &mut self.levels;
		let index = *self
			.level_index
			.entry(lighting.to_bits())
			.or_insert_with(|| {
				levels.push(ShadeLevel {
					lighting,
					shaded: [UNSHADED; 256],
				});
				levels.len() - 1
			});
		&mut self.levels[index]
	}
}

impl ShadeLevel {
	/// Convert sRGB to linear, apply lighting, then convert back to sRGB.
	#[inline]
	fn shade(&mut self, channel: u8) -> u8 {
		let cached = self.shaded[channel as usize];
		if cached != UNSHADED {
			return cached as u8;
		}
		let linear = linear_channels()[channel as usize] * self.lighting;
		let shaded = (linear.powf(1.0 / GAMMA) * 255.0).clamp(0.0, 255.0) as u8;
		self.shaded[channel as usize] = u16::from(shaded);
		shaded
	}
}

/// Everything needed to colour a pixel that is constant across a face.
pub(crate) struct FaceSampler<'a> {
	pub texture: &'a Texture,
	pub uv_face: &'a UvFace,
	pub face_width: f32,
	pub face_height: f32,
	pub tint: Option<&'a TintGradient>,
	pub bilinear_filtering: bool,
}

impl FaceSampler<'_> {
	#[inline]
	fn sample(&self, u: f32, v: f32) -> Rgba<u8> {
		let pixel = if self.bilinear_filtering {
			sample_face_texture_bilinear(
				self.texture,
				self.uv_face,
				self.face_width,
				self.face_height,
				u,
				v,
			)
		} else {
			sample_face_texture(
				self.texture,
				self.uv_face,
				self.face_width,
				self.face_height,
				u,
				v,
			)
		};
		match self.tint {
			Some(tint) => apply_tint(pixel, tint),
			None => pixel,
		}
	}
}

/// Render a triangle with depth buffer, texture sampling, and optional shading
pub(crate) fn render_triangle_tinted(
	image: &mut RgbaImage,
	depth_buffer: &mut [f32],
	output_width: u32,
	vertices: [(f32, f32, f32); 3],
	uvs: [(f32, f32); 3],
	sampler: &FaceSampler,
	mut shade: Option<&mut ShadeLevel>,
) {
	let [(x0, y0, z0), (x1, y1, z1), (x2, y2, z2)] = vertices;
	let [(uv0_u, uv0_v), (uv1_u, uv1_v), (uv2_u, uv2_v)] = uvs;

	let Some(barycentric) = Barycentric::new(x0, y0, x1, y1, x2, y2) else {
		return;
	};

	let min_x = x0.min(x1).min(x2).max(0.0) as u32;
	let max_x = x0.max(x1).max(x2).min(image.width() as f32) as u32;
	let min_y = y0.min(y1).min(y2).max(0.0) as u32;
	let max_y = y0.max(y1).max(y2).min(image.height() as f32) as u32;

	for y in min_y..=max_y.min(image.height() - 1) {
		let py = y as f32 + 0.5;
		for x in min_x..=max_x.min(image.width() - 1) {
			let px = x as f32 + 0.5;

			let (bary_u, bary_v, bary_w) = barycentric.coords(px, py);
			let inside = bary_u >= 0.0 && bary_v >= 0.0 && bary_w >= 0.0;
			if !inside {
				continue;
			}

			// Interpolate depth and check buffer
			let depth = bary_w * z0 + bary_v * z1 + bary_u * z2;
			let buffer_index = (y * output_width + x) as usize;
			let closer = depth < depth_buffer[buffer_index] - DEPTH_BIAS;
			if !closer {
				continue;
			}

			// Interpolate UV coordinates (vertex weights: v0=w, v1=v, v2=u)
			let tex_u = bary_w * uv0_u + bary_v * uv1_u + bary_u * uv2_u;
			let tex_v = bary_w * uv0_v + bary_v * uv1_v + bary_u * uv2_v;

			let mut pixel = sampler.sample(tex_u, tex_v);

			// Shade after tinting; alpha is left untouched
			if let Some(level) = shade.as_deref_mut() {
				pixel = Rgba([
					level.shade(pixel[0]),
					level.shade(pixel[1]),
					level.shade(pixel[2]),
					pixel[3],
				]);
			}

			if pixel[3] > 0 {
				depth_buffer[buffer_index] = depth;
				image.put_pixel(x, y, pixel);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The shading the rasterizer applied per pixel before results were memoised.
	fn shade_directly(channel: u8, lighting: f32) -> u8 {
		let linear = (channel as f32 / 255.0).powf(2.2) * lighting;
		(linear.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8
	}

	#[test]
	fn memoised_shading_matches_direct_computation() {
		let mut cache = ShadeCache::default();
		for lighting in [0.0, 0.35, 0.85, 0.9731, 1.0] {
			// Twice per level, so both the computing and the cached path are checked.
			for _ in 0..2 {
				for channel in 0..=255u8 {
					assert_eq!(
						cache.level(lighting).shade(channel),
						shade_directly(channel, lighting),
						"channel {} at light level {}",
						channel,
						lighting
					);
				}
			}
		}
	}

	#[test]
	fn light_levels_do_not_share_results() {
		let mut cache = ShadeCache::default();
		let lit = cache.level(1.0).shade(200);
		let dim = cache.level(0.5).shade(200);
		assert!(dim < lit);
		assert_eq!(cache.level(1.0).shade(200), lit);
	}
}
