//! Renders a fixed set of skins from the locally extracted Hytale assets, reporting per-stage
//! timings and writing each render (plus the asset list it needed) for regression comparison.
//!
//! Usage: cargo run --release --manifest-path src/hytale-skin-renderer/Cargo.toml \
//!        --example render_bench -- <out_dir> [iterations]

use hytale_skin_renderer::asset_provider::AssetProvider;
use hytale_skin_renderer::render_pipeline::RenderContext;
use hytale_skin_renderer::{camera, output, renderer, texture, Error, Result};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const ASSET_ROOT: &str = "assets/Common";

/// Serves provider paths from the extracted asset tree, remembering what was asked for.
struct RecordingProvider {
	root: PathBuf,
	// Files stay in memory after the first read so timings exclude disk I/O, as in a worker.
	loaded: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl RecordingProvider {
	fn new(root: &Path) -> Self {
		RecordingProvider {
			root: root.to_path_buf(),
			loaded: RefCell::new(BTreeMap::new()),
		}
	}

	fn text(&self, path: &str) -> Result<String> {
		String::from_utf8(self.load_bytes(path)?).map_err(|e| Error::InvalidData(e.to_string()))
	}
}

impl AssetProvider for RecordingProvider {
	fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
		if let Some(bytes) = self.loaded.borrow().get(path) {
			return Ok(bytes.clone());
		}
		let relative = path
			.strip_prefix("assets/Common/")
			.or_else(|| path.strip_prefix("assets/"))
			.unwrap_or(path);
		// Game files live under Common/, the bundled metadata JSON sits beside it.
		let candidates = [
			self.root.join("Common").join(relative),
			self.root.join(relative),
		];
		for candidate in candidates {
			if let Ok(bytes) = std::fs::read(&candidate) {
				self.loaded
					.borrow_mut()
					.insert(path.to_string(), bytes.clone());
				return Ok(bytes);
			}
		}
		Err(Error::InvalidData(format!("Asset not provided: {}", path)))
	}
}

struct Scenario {
	name: &'static str,
	base_texture: &'static str,
	skin: &'static str,
}

const SCENARIOS: &[Scenario] = &[
	Scenario {
		name: "basic",
		base_texture: "Characters/Player_Textures/Player_Greyscale.png",
		skin: r#"{"skin":{"bodyCharacteristic":"Default.10","underwear":"Suit.Blue","face":"Face_Neutral","ears":"Default","mouth":"Mouth_Default","haircut":"Morning.BrownDark","eyebrows":"Medium.BrownDark","eyes":"Plain_Eyes.Brown"}}"#,
	},
	Scenario {
		name: "full",
		base_texture: "Characters/Player_Textures/Player_Muscular_Greyscale.png",
		skin: r#"{"skin":{"bodyCharacteristic":"Muscular.14","underwear":"Boxer.Red","face":"Face_Scar","ears":"Elf_Ears","mouth":"Mouth_Thin","haircut":"Bangs.Black","facialHair":"Medium.Black","eyebrows":"Thick.Black","eyes":"Large_Eyes.Honey","pants":"LeatherPants.Orange_Tan","overpants":"Socks_Plain.Blue","undertop":"Wide_Neck_Shirt.Green","overtop":"PuffyJacket.Red.Unzipped","shoes":"BasicBoots.Brown","headAccessory":"Goggles.Yellow","faceAccessory":"Glasses.Purple","earAccessory":"EarHoops.Gold_Red.Both","gloves":"BasicGloves_Basic.Green","cape":"Cape_Royal_Emissary.Blue.NoNeck"}}"#,
	},
	// Exercises renamed-part fallbacks and hair culling under a fully covering accessory.
	Scenario {
		name: "covered",
		base_texture: "Characters/Player_Textures/Player_Greyscale.png",
		skin: r#"{"skin":{"bodyCharacteristic":"Default.12","underwear":"Bra.Turquoise","face":"Face_Neutral_Freckles","ears":"Default","mouth":"Mouth_Long","haircut":"BobCut.Brown","eyebrows":"Thin.Brown","eyes":"Almond_Eyes.Red","pants":"ApprenticePants.Black","undertop":"VNeck_Shirt.Lime","shoes":"QuiltedBoots.Black","headAccessory":"Hoodie.Purple","earAccessory":"SimpleEarring.Silver_Blue.Left","cape":"Cape_Trimmed.Red"}}"#,
	},
	Scenario {
		name: "particles",
		base_texture: "Characters/Player_Textures/Player_Goblin_Greyscale.png",
		skin: r#"{"skin":{"bodyCharacteristic":"Goblin.16","underwear":"Suit.Green","face":"Face_Sunken","ears":"Ogre_Ears","mouth":"Mouth_Tiny","haircut":"Quiff.BrownLight","eyebrows":"Bushy.BrownLight","eyes":"Shiny_Eyes.Yellow","headAccessory":"GoblinTrash_BananaPeel.Rotten"}}"#,
	},
];

/// View type and how many times taller than wide its render is.
const VIEWS: &[(&str, u32)] = &[
	("headshot", 1),
	("isometric_head", 1),
	("player_bust", 1),
	("full_body_front", 2),
];

const SIZES: &[u32] = &[64, 180, 300];

#[derive(Default, Clone, Copy)]
struct Stages {
	skin: Duration,
	raster: Duration,
	encode: Duration,
}

impl Stages {
	fn total(&self) -> Duration {
		self.skin + self.raster + self.encode
	}
}

/// The same steps as `RenderContext::render_png`, timed separately.
fn render_once(
	context: &RenderContext,
	assets: &RecordingProvider,
	scenario: &Scenario,
	view: &str,
	width: u32,
	height: u32,
) -> Result<(Vec<u8>, Stages)> {
	let mut stages = Stages::default();

	let started = Instant::now();
	let body = context.dress(scenario.skin, assets)?;
	let base_texture = texture::Texture::from_bytes(
		&assets.load_bytes(&format!("{}/{}", ASSET_ROOT, scenario.base_texture))?,
	)?;
	stages.skin = started.elapsed();

	let started = Instant::now();
	let image = renderer::render_scene_tinted(
		&body.faces,
		&base_texture,
		camera::for_view_type(view).as_ref(),
		width,
		height,
		&body.tint_config,
		&body.particles,
	)?;
	stages.raster = started.elapsed();

	let started = Instant::now();
	let png = output::export_png_bytes(&image)?;
	stages.encode = started.elapsed();

	Ok((png, stages))
}

fn median(mut samples: Vec<Duration>) -> f64 {
	samples.sort();
	samples[samples.len() / 2].as_secs_f64() * 1000.0
}

fn load_context(root: &Path) -> Result<RenderContext> {
	let shared = RecordingProvider::new(root);
	let model_json = shared.text("assets/Common/Characters/Player.blockymodel")?;
	let animation_json =
		shared.text("assets/Common/Characters/Animations/Default/Idle.blockyanim")?;
	RenderContext::new(&model_json, &animation_json, Box::new(shared), ASSET_ROOT)
}

fn main() -> Result<()> {
	let args: Vec<String> = std::env::args().collect();
	let Some(out_dir) = args.get(1).map(PathBuf::from) else {
		eprintln!("Usage: render_bench <out_dir> [iterations]");
		std::process::exit(1);
	};
	let iterations: usize = args.get(2).and_then(|n| n.parse().ok()).unwrap_or(15);

	let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/hytale");
	if !root.join("Common/Characters/Player.blockymodel").exists() {
		eprintln!("Hytale assets are not extracted; see assets/README.md");
		std::process::exit(1);
	}
	std::fs::create_dir_all(&out_dir).map_err(Error::Io)?;

	// The first load also warms the file cache, so time a second one.
	load_context(&root)?;
	let started = Instant::now();
	let context = load_context(&root)?;
	println!(
		"context load (once per isolate, includes file reads): {:.3} ms",
		started.elapsed().as_secs_f64() * 1000.0
	);

	println!(
		"{:<34} {:>8} {:>8} {:>8} {:>8}",
		"scenario", "skin", "raster", "encode", "total"
	);
	let mut manifest = Vec::new();
	let mut grand_total = 0.0;
	for scenario in SCENARIOS {
		let assets = RecordingProvider::new(&root);
		for (view, height_factor) in VIEWS {
			for size in SIZES {
				let (width, height) = (*size, size * height_factor);
				let mut samples = Vec::with_capacity(iterations);
				let mut png = Vec::new();
				for _ in 0..iterations {
					let (bytes, stages) =
						render_once(&context, &assets, scenario, view, width, height)?;
					png = bytes;
					samples.push(stages);
				}
				let name = format!("{}-{}-{}", scenario.name, view, size);
				std::fs::write(out_dir.join(format!("{}.png", name)), &png).map_err(Error::Io)?;

				let stage =
					|pick: fn(&Stages) -> Duration| median(samples.iter().map(pick).collect());
				let total = stage(Stages::total);
				grand_total += total;
				println!(
					"{:<34} {:>8.3} {:>8.3} {:>8.3} {:>8.3}",
					name,
					stage(|s| s.skin),
					stage(|s| s.raster),
					stage(|s| s.encode),
					total
				);
			}
		}
		// Shared JSON (e.g. the particle manifest) is loaded with the context, not per skin.
		let skin_assets: Vec<String> = assets
			.loaded
			.borrow()
			.keys()
			.filter(|path| !path.ends_with(".json"))
			.cloned()
			.collect();
		manifest.push(serde_json::json!({
			"name": scenario.name,
			"baseTexture": scenario.base_texture,
			"skin": scenario.skin,
			"assets": skin_assets,
		}));
	}
	println!("sum of medians: {:.3} ms", grand_total);

	let manifest_json =
		serde_json::to_string_pretty(&manifest).map_err(|e| Error::InvalidData(e.to_string()))?;
	std::fs::write(out_dir.join("manifest.json"), manifest_json).map_err(Error::Io)?;
	Ok(())
}
