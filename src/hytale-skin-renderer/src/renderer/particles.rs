//! Alpha-blended billboard pass for particle sprites, drawn after the opaque scene.

use crate::camera::CameraProjection;
use crate::models::Vector3;
use crate::particles::{ParticleSprite, SpriteFacing};
use glam::{Vec2, Vec3, Vec4Swizzles};
use image::RgbaImage;

pub(super) fn render_particles(
	image: &mut RgbaImage,
	depth_buffer: &[f32],
	sprites: &[ParticleSprite],
	camera: &dyn CameraProjection,
) {
	let (width, height) = image.dimensions();
	let vp = camera.view_projection_matrix(width, height);
	let view = camera.view_matrix();
	let camera_right = view.row(0).xyz().normalize_or_zero();
	let camera_up = view.row(1).xyz().normalize_or_zero();

	let depth_of = |p: Vec3| {
		camera.calculate_depth(Vector3 {
			x: p.x,
			y: p.y,
			z: p.z,
		})
	};

	// Blending is order dependent, so composite far sprites first.
	let mut ordered: Vec<(f32, &ParticleSprite)> =
		sprites.iter().map(|s| (depth_of(s.center), s)).collect();
	ordered.sort_by(|a, b| b.0.total_cmp(&a.0));

	for (depth, sprite) in ordered {
		let (right, up) = match sprite.facing {
			SpriteFacing::Camera => (camera_right, camera_up),
			SpriteFacing::Upright => {
				let flat_right = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize_or_zero();
				if flat_right == Vec3::ZERO {
					(camera_right, camera_up)
				} else {
					(flat_right, Vec3::Y)
				}
			}
		};
		let half = sprite.size * 0.5;

		// Corners in quad space; UV (0,0) is the top-left of the frame.
		let corners = [(-1.0, 1.0), (1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
		let mut screen = [Vec2::ZERO; 4];
		let mut visible = true;
		for (i, (cx, cy)) in corners.iter().enumerate() {
			let world = sprite.center + right * (cx * half.x) + up * (cy * half.y);
			let clip = vp * world.extend(1.0);
			if clip.w <= 0.0 {
				visible = false;
				break;
			}
			let ndc = clip.xy() / clip.w;
			screen[i] = Vec2::new(
				(ndc.x + 1.0) * 0.5 * width as f32,
				(1.0 - ndc.y) * 0.5 * height as f32,
			);
		}
		if !visible {
			continue;
		}

		let uvs = [
			Vec2::new(0.0, 0.0),
			Vec2::new(1.0, 0.0),
			Vec2::new(1.0, 1.0),
			Vec2::new(0.0, 1.0),
		];
		for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
			draw_triangle(
				image,
				depth_buffer,
				sprite,
				depth,
				[screen[a], screen[b], screen[c]],
				[uvs[a], uvs[b], uvs[c]],
			);
		}
	}
}

fn draw_triangle(
	image: &mut RgbaImage,
	depth_buffer: &[f32],
	sprite: &ParticleSprite,
	depth: f32,
	points: [Vec2; 3],
	uvs: [Vec2; 3],
) {
	let (width, height) = image.dimensions();
	let [p0, p1, p2] = points;
	let area = (p1 - p0).perp_dot(p2 - p0);
	if area.abs() < f32::EPSILON {
		return;
	}

	let min = p0.min(p1).min(p2).floor().max(Vec2::ZERO);
	let max = p0
		.max(p1)
		.max(p2)
		.ceil()
		.min(Vec2::new(width as f32 - 1.0, height as f32 - 1.0));
	if min.x > max.x || min.y > max.y {
		return;
	}
	let (frame_x, frame_y, frame_w, frame_h) = sprite.frame;
	let [r, g, b, opacity] = sprite.color;

	for y in min.y as u32..=max.y as u32 {
		for x in min.x as u32..=max.x as u32 {
			let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
			let w0 = (p2 - p1).perp_dot(p - p1) / area;
			let w1 = (p0 - p2).perp_dot(p - p2) / area;
			let w2 = 1.0 - w0 - w1;
			if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
				continue;
			}
			let index = (y * width + x) as usize;
			if depth >= depth_buffer[index] {
				continue;
			}

			let uv = uvs[0] * w0 + uvs[1] * w1 + uvs[2] * w2;
			let texel = sprite.texture.sample_pixel(
				frame_x as f32 + uv.x.clamp(0.0, 0.999) * frame_w as f32,
				frame_y as f32 + uv.y.clamp(0.0, 0.999) * frame_h as f32,
			);
			let src_alpha = texel[3] as f32 / 255.0 * opacity;
			if src_alpha <= 0.0 {
				continue;
			}
			let src = [
				texel[0] as f32 * r,
				texel[1] as f32 * g,
				texel[2] as f32 * b,
			];

			// Straight-alpha "over" so sprites also composite onto the transparent background.
			let dst = image.get_pixel_mut(x, y);
			let dst_alpha = dst[3] as f32 / 255.0;
			let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
			for c in 0..3 {
				let blended = (src[c] * src_alpha + dst[c] as f32 * dst_alpha * (1.0 - src_alpha))
					/ out_alpha;
				dst[c] = blended.round().clamp(0.0, 255.0) as u8;
			}
			dst[3] = (out_alpha * 255.0).round() as u8;
		}
	}
}
