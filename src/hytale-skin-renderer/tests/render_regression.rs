//! End-to-end renders of a synthetic scene, compared against stored images.
//!
//! The scene needs no game assets, yet touches everything a real character render does: tinting
//! chosen by node name, per-face textures and tints, partial texture layouts, mirrored/rotated
//! UVs, negative stretch, double-sided quads, frustum clipping, lighting and particles.
//!
//! Regenerate the stored images after an intentional change with:
//! UPDATE_GOLDEN=1 cargo test --manifest-path src/hytale-skin-renderer/Cargo.toml --test render_regression

use glam::{Vec2, Vec3};
use hytale_skin_renderer::camera::{Camera, CameraProjection, PerspectiveCamera};
use hytale_skin_renderer::particles::{ParticleSprite, SpriteFacing};
use hytale_skin_renderer::renderer::{
	render_scene_tinted, render_scene_tinted_with_config, LightConfig, RenderConfig,
	RenderableFace, TintConfig,
};
use hytale_skin_renderer::texture::{Texture, TintGradient};
use hytale_skin_renderer::{geometry, models, scene};
use image::{DynamicImage, Rgba, RgbaImage};
use std::path::PathBuf;
use std::sync::Arc;

/// Channel difference below which two pixels count as equal.
/// Shading goes through `powf`, whose last bit varies between platforms' maths libraries.
const CHANNEL_TOLERANCE: u8 = 2;

/// Share of pixels allowed to differ outright.
/// A pixel centre sitting on a triangle edge can land on either side of it across platforms.
const MAX_MISMATCHED_SHARE: f64 = 0.005;

fn uv(x: u32, y: u32, mirror_x: bool, angle: u32) -> String {
	format!(
		r#"{{"offset":{{"x":{},"y":{}}},"mirror":{{"x":{},"y":false}},"angle":{}}}"#,
		x, y, mirror_x, angle
	)
}

#[allow(clippy::too_many_arguments)]
fn node(
	name: &str,
	position: (f32, f32, f32),
	orientation: (f32, f32, f32, f32),
	shape_type: &str,
	settings: &str,
	stretch_x: f32,
	double_sided: bool,
	layout: &[(&str, String)],
) -> String {
	let layout: Vec<String> = layout
		.iter()
		.map(|(face, uv)| format!(r#""{}":{}"#, face, uv))
		.collect();
	format!(
		r#"{{"id":"{name}","name":"{name}",
		"position":{{"x":{},"y":{},"z":{}}},
		"orientation":{{"x":{},"y":{},"z":{},"w":{}}},
		"shape":{{"type":"{shape_type}","settings":{settings},
			"stretch":{{"x":{stretch_x},"y":1,"z":1}},
			"doubleSided":{double_sided},
			"textureLayout":{{{}}}}}}}"#,
		position.0,
		position.1,
		position.2,
		orientation.0,
		orientation.1,
		orientation.2,
		orientation.3,
		layout.join(","),
	)
}

fn scene_model() -> models::BlockyModel {
	let all_faces = |x: u32, y: u32| {
		vec![
			("front", uv(x, y, false, 0)),
			("back", uv(x + 16, y, false, 0)),
			("left", uv(x, y + 16, false, 0)),
			("right", uv(x + 16, y + 16, false, 0)),
			("top", uv(x + 32, y, false, 0)),
			("bottom", uv(x + 32, y + 16, false, 0)),
		]
	};
	let nodes = [
		// Turned so three faces, each with its own light level, are visible.
		node(
			"Head",
			(0.0, 0.0, 0.0),
			(0.0, 0.258_819, 0.0, 0.965_925_8),
			"box",
			r#"{"size":{"x":16,"y":16,"z":16}}"#,
			1.0,
			false,
			&all_faces(0, 0),
		),
		// Only some faces have a layout; the rest must be skipped rather than drawn.
		node(
			"HairBase",
			(0.0, 11.0, 0.0),
			(0.0, 0.258_819, 0.0, 0.965_925_8),
			"box",
			r#"{"size":{"x":18,"y":6,"z":18}}"#,
			1.0,
			false,
			&[
				("front", uv(4, 34, false, 0)),
				("top", uv(24, 34, false, 0)),
				("right", uv(44, 34, false, 0)),
			],
		),
		// "Background" in the name marks an eye white, which is never tinted.
		node(
			"R-Eye-Background",
			(-4.0, 2.0, 9.5),
			(0.0, 0.0, 0.0, 1.0),
			"quad",
			r#"{"size":{"x":4,"y":4},"normal":"+Z"}"#,
			1.0,
			false,
			&[("front", uv(50, 2, false, 0))],
		),
		node(
			"L-Eye",
			(4.0, 2.0, 9.5),
			(0.0, 0.0, 0.0, 1.0),
			"quad",
			r#"{"size":{"x":4,"y":4},"normal":"+Z"}"#,
			1.0,
			false,
			&[("front", uv(50, 8, true, 0))],
		),
		node(
			"Cape",
			(0.0, -14.0, -9.0),
			(0.130_526, 0.0, 0.0, 0.991_445),
			"quad",
			r#"{"size":{"x":20,"y":24},"normal":"-Z"}"#,
			1.0,
			true,
			&[("back", uv(2, 40, false, 0))],
		),
		// Negative stretch flips the winding; rotated and mirrored UVs take the other sampling paths.
		node(
			"Flipped",
			(-15.0, -7.0, 4.0),
			(0.0, -0.382_683, 0.0, 0.923_88),
			"box",
			r#"{"size":{"x":6,"y":6,"z":6}}"#,
			-1.0,
			false,
			&[
				("front", uv(30, 44, true, 90)),
				("left", uv(40, 44, false, 180)),
				("right", uv(40, 54, true, 270)),
				("top", uv(52, 54, false, 90)),
			],
		),
		node(
			"Accessory",
			(12.0, 10.0, 2.0),
			(0.0, 0.0, 0.258_819, 0.965_925_8),
			"box",
			r#"{"size":{"x":8,"y":4,"z":8}}"#,
			1.0,
			false,
			&all_faces(0, 0),
		),
		// Reaches well outside the view, so its faces are clipped against the frustum.
		node(
			"Slab",
			(30.0, -24.0, 0.0),
			(0.0, 0.0, 0.0, 1.0),
			"box",
			r#"{"size":{"x":60,"y":8,"z":12}}"#,
			1.0,
			false,
			&all_faces(8, 8),
		),
	];
	let json = format!(r#"{{"nodes":[{}]}}"#, nodes.join(","));
	models::parse_blockymodel(&json).expect("synthetic model parses")
}

/// Greys (tintable), coloured texels (kept as they are), and transparent or translucent holes.
fn patterned_texture(size: u32, salt: u32) -> Texture {
	let image = RgbaImage::from_fn(size, size, |x, y| {
		let mix = x * 7 + y * 3 + salt;
		if mix.is_multiple_of(11) {
			Rgba([0, 0, 0, 0])
		} else if mix.is_multiple_of(13) {
			Rgba([200, 200, 200, 128])
		} else if (x + y + salt).is_multiple_of(5) {
			Rgba([(x * 4) as u8, 255 - (y * 4) as u8, 90, 255])
		} else {
			let grey = ((x * 9 + y * 5 + salt) % 256) as u8;
			Rgba([grey, grey, grey, 255])
		}
	});
	Texture::from_image(DynamicImage::ImageRgba8(image))
}

fn gradient(colours: &[&str]) -> TintGradient {
	let colours: Vec<String> = colours.iter().map(|colour| colour.to_string()).collect();
	TintGradient::from_hex_colors(&colours)
}

struct SyntheticScene {
	faces: Vec<RenderableFace>,
	texture: Texture,
	tint_config: TintConfig,
	particles: Vec<ParticleSprite>,
}

fn synthetic_scene() -> SyntheticScene {
	let graph = scene::SceneGraph::from_blockymodel(&scene_model()).expect("scene builds");
	let accessory_texture = Arc::new(patterned_texture(48, 3));
	let accessory_tint = Arc::new(gradient(&["#1d3b8a", "#9fd6ff"]).with_inverted(true));

	let mut faces = Vec::new();
	for (node, transform) in graph.get_visible_shapes() {
		let shape = node.shape.as_ref().expect("visible nodes have a shape");
		let is_accessory = node.name == "Accessory";
		for face in geometry::generate_geometry(shape, transform) {
			faces.push(RenderableFace {
				face,
				transform,
				shape: Some(shape.clone()),
				node_name: Some(node.name.clone()),
				texture: is_accessory.then(|| Arc::clone(&accessory_texture)),
				tint: is_accessory.then(|| Arc::clone(&accessory_tint)),
			});
		}
	}

	let mut tint_config = TintConfig::with_skin(gradient(&["#3b1f14", "#f2c9a5"]));
	tint_config.hair = Some(gradient(&["#2a1608", "#c98a3c", "#fff1c9"]));
	tint_config.eyes = Some(gradient(&["#0b3d2e", "#45e0a8"]));
	tint_config.cape = Some(gradient(&["#5a0f1c", "#ff6b81"]).with_brightness(0.8));

	let sprite_texture = Arc::new(patterned_texture(16, 1));
	let particles = vec![
		ParticleSprite {
			center: Vec3::new(-6.0, 6.0, 14.0),
			size: Vec2::new(10.0, 10.0),
			facing: SpriteFacing::Camera,
			color: [1.0, 0.8, 0.6, 0.5],
			texture: Arc::clone(&sprite_texture),
			frame: (0, 0, 8, 8),
		},
		// Sits behind the head, so the depth buffer hides part of it.
		ParticleSprite {
			center: Vec3::new(10.0, -4.0, -12.0),
			size: Vec2::new(-14.0, 14.0),
			facing: SpriteFacing::Upright,
			color: [0.6, 1.0, 0.9, 0.9],
			texture: sprite_texture,
			frame: (8, 8, 8, 8),
		},
	];

	SyntheticScene {
		faces,
		texture: patterned_texture(64, 0),
		tint_config,
		particles,
	}
}

fn golden_path(name: &str) -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests/golden")
		.join(format!("{}.png", name))
}

fn assert_matches_golden(name: &str, rendered: &RgbaImage) {
	let path = golden_path(name);
	if std::env::var_os("UPDATE_GOLDEN").is_some() {
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		rendered.save(&path).unwrap();
		return;
	}

	let golden = image::open(&path)
		.unwrap_or_else(|error| panic!("missing golden image {}: {}", path.display(), error))
		.into_rgba8();
	assert_eq!(
		rendered.dimensions(),
		golden.dimensions(),
		"{}: size changed",
		name
	);

	let mismatched = rendered
		.pixels()
		.zip(golden.pixels())
		.filter(|(rendered, golden)| {
			rendered
				.0
				.iter()
				.zip(golden.0.iter())
				.any(|(a, b)| a.abs_diff(*b) > CHANNEL_TOLERANCE)
		})
		.count();
	let allowed = (f64::from(golden.width() * golden.height()) * MAX_MISMATCHED_SHARE) as usize;
	assert!(
		mismatched <= allowed,
		"{}: {} pixels differ from the golden image (at most {} allowed)",
		name,
		mismatched,
		allowed
	);
}

fn render(
	scene: &SyntheticScene,
	camera: &dyn CameraProjection,
	width: u32,
	height: u32,
) -> RgbaImage {
	render_scene_tinted(
		&scene.faces,
		&scene.texture,
		camera,
		width,
		height,
		&scene.tint_config,
		&scene.particles,
	)
	.expect("scene renders")
}

#[test]
fn perspective_render_matches_golden() {
	let scene = synthetic_scene();
	let camera = PerspectiveCamera::new(Vec3::new(28.0, 22.0, 62.0), Vec3::ZERO, 38.0);
	assert_matches_golden("perspective", &render(&scene, &camera, 96, 96));
}

#[test]
fn orthographic_render_matches_golden() {
	let scene = synthetic_scene();
	let camera = Camera::new(
		Vec3::new(-30.0, 15.0, 90.0),
		Vec3::new(0.0, -4.0, 0.0),
		64.0,
	);
	assert_matches_golden("orthographic", &render(&scene, &camera, 64, 128));
}

#[test]
fn unlit_bilinear_render_matches_golden() {
	let scene = synthetic_scene();
	let camera = PerspectiveCamera::new(Vec3::new(-24.0, 10.0, 58.0), Vec3::ZERO, 40.0);
	let config = RenderConfig {
		bilinear_filtering: true,
		blur_amount: 0.0,
		light_config: LightConfig {
			enabled: false,
			..LightConfig::default()
		},
	};
	let rendered = render_scene_tinted_with_config(
		&scene.faces,
		&scene.texture,
		&camera,
		80,
		80,
		&scene.tint_config,
		&scene.particles,
		config,
	)
	.expect("scene renders");
	assert_matches_golden("unlit_bilinear", &rendered);
}

#[test]
fn render_draws_the_scene() {
	// Guards the golden comparison itself: a blank image would match a blank golden.
	let scene = synthetic_scene();
	let camera = PerspectiveCamera::new(Vec3::new(28.0, 22.0, 62.0), Vec3::ZERO, 38.0);
	let rendered = render(&scene, &camera, 96, 96);
	let drawn = rendered.pixels().filter(|pixel| pixel[3] > 0).count();
	assert!(drawn > 2000, "only {} of 9216 pixels were drawn", drawn);
	assert!(
		drawn < 9216,
		"the background should stay transparent somewhere"
	);
}
