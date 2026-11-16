//! `RenderContext` is built once and then serves every render, so these tests check that a
//! render depends only on the skin it was asked for and not on what was rendered before it.
//!
//! The cosmetic registry is the real bundled JSON; models and textures are synthetic stand-ins
//! placed at the paths the registry points to, so no extracted game assets are needed.

use hytale_skin_renderer::asset_provider::{FileAssetProvider, MemoryAssetProvider};
use hytale_skin_renderer::render_pipeline::RenderContext;
use image::{DynamicImage, Rgba, RgbaImage};

const PLAYER_MODEL: &str = r#"{"nodes":[{"id":"1","name":"Head",
	"position":{"x":0,"y":99,"z":0},"orientation":{"x":0,"y":0,"z":0,"w":1},
	"shape":{"type":"box","settings":{"size":{"x":16,"y":16,"z":16}},
		"textureLayout":{"front":{"offset":{"x":0,"y":0},"mirror":{"x":false,"y":false},"angle":0}}}}]}"#;

const IDLE_ANIMATION: &str = r#"{"duration":60,"nodeAnimations":{}}"#;

/// Stand-in for the "Morning" haircut: a slab resting on top of the head.
const HAIRCUT_MODEL: &str = r#"{"nodes":[{"id":"1","name":"Head",
	"position":{"x":0,"y":0,"z":0},"orientation":{"x":0,"y":0,"z":0,"w":1},
	"shape":{"type":"none","settings":{}},
	"children":[{"id":"2","name":"HairBase",
		"position":{"x":0,"y":8,"z":0},"orientation":{"x":0,"y":0,"z":0,"w":1},
		"shape":{"type":"box","settings":{"size":{"x":18,"y":6,"z":18}},
			"textureLayout":{"front":{"offset":{"x":0,"y":0},"mirror":{"x":false,"y":false},"angle":0}}}}]}]}"#;

const BALD_SKIN: &str = r#"{"skin":{"bodyCharacteristic":"Default.10"}}"#;
const HAIRED_SKIN: &str =
	r#"{"skin":{"bodyCharacteristic":"Default.10","haircut":"Morning.BrownDark"}}"#;

fn png(image: RgbaImage) -> Vec<u8> {
	let mut bytes = Vec::new();
	DynamicImage::ImageRgba8(image)
		.write_to(
			&mut std::io::Cursor::new(&mut bytes),
			image::ImageFormat::Png,
		)
		.unwrap();
	bytes
}

fn greyscale_texture() -> Vec<u8> {
	png(RgbaImage::from_fn(32, 32, |x, y| {
		let grey = (40 + x * 5 + y) as u8;
		Rgba([grey, grey, grey, 255])
	}))
}

fn gradient(from: [u8; 3], to: [u8; 3]) -> Vec<u8> {
	png(RgbaImage::from_fn(256, 1, |x, _| {
		let mix = |a: u8, b: u8| (u32::from(a) + (u32::from(b) - u32::from(a)) * x / 255) as u8;
		Rgba([
			mix(from[0], to[0]),
			mix(from[1], to[1]),
			mix(from[2], to[2]),
			255,
		])
	}))
}

fn context() -> RenderContext {
	let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/hytale");
	RenderContext::new(
		PLAYER_MODEL,
		IDLE_ANIMATION,
		Box::new(FileAssetProvider::new(root)),
		root,
	)
	.expect("context builds from the bundled registry")
}

/// The per-render assets a worker would fetch for these skins.
fn skin_assets() -> MemoryAssetProvider {
	let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/hytale");
	let assets = vec![
		(
			format!("{}/TintGradients/Skin_Tones/10.png", root),
			gradient([60, 30, 20], [240, 200, 170]),
		),
		(
			format!("{}/TintGradients/Hair/Brown_Dark.png", root),
			gradient([20, 8, 4], [120, 60, 30]),
		),
		(
			"assets/Common/Characters/Haircuts/Morning.blockymodel".to_string(),
			HAIRCUT_MODEL.as_bytes().to_vec(),
		),
		(
			"assets/Common/Characters/Haircuts/Morning_Greyscale.png".to_string(),
			greyscale_texture(),
		),
	];
	let (paths, bytes) = assets.into_iter().unzip();
	MemoryAssetProvider::new(paths, bytes).unwrap()
}

fn render(
	context: &RenderContext,
	skin: &str,
	view_type: &str,
	width: u32,
	height: u32,
) -> RgbaImage {
	let png = context
		.render_png(
			skin,
			&greyscale_texture(),
			&skin_assets(),
			view_type,
			width,
			height,
		)
		.expect("skin renders");
	image::load_from_memory(&png)
		.expect("output is a readable image")
		.into_rgba8()
}

#[test]
fn renders_requested_size_with_the_character_in_view() {
	let context = context();
	let rendered = render(&context, BALD_SKIN, "headshot", 96, 96);
	assert_eq!(rendered.dimensions(), (96, 96));
	// The head faces the headshot camera, so the centre is skin and the corner is background.
	assert_eq!(rendered.get_pixel(48, 48)[3], 255);
	assert_eq!(rendered.get_pixel(0, 0)[3], 0);

	let body = render(&context, BALD_SKIN, "full_body_front", 40, 80);
	assert_eq!(body.dimensions(), (40, 80));
}

#[test]
fn cosmetics_change_the_render() {
	let context = context();
	let bald = render(&context, BALD_SKIN, "headshot", 96, 96);
	let haired = render(&context, HAIRED_SKIN, "headshot", 96, 96);
	assert!(bald != haired, "the haircut should be visible");
}

#[test]
fn earlier_renders_do_not_affect_later_ones() {
	let reused = context();
	let bald_first = render(&reused, BALD_SKIN, "headshot", 96, 96);
	let haired = render(&reused, HAIRED_SKIN, "headshot", 96, 96);
	let bald_again = render(&reused, BALD_SKIN, "headshot", 96, 96);
	assert!(
		bald_first == bald_again,
		"a cosmetic leaked into a later render"
	);

	// A context that has rendered other skins must agree with one that has rendered nothing.
	let fresh = render(&context(), HAIRED_SKIN, "headshot", 96, 96);
	assert!(haired == fresh, "render depends on the context's history");
}

#[test]
fn unreadable_skin_config_is_an_error() {
	let result = context().render_png(
		"not json",
		&greyscale_texture(),
		&skin_assets(),
		"headshot",
		64,
		64,
	);
	assert!(result.is_err());
}
