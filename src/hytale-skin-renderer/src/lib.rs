//! Hytale Blockymodel Renderer
//!
//! A Rust library for parsing and rendering Hytale blockymodel files to PNG images.

pub mod animation;
pub mod asset_provider;
pub mod camera;
pub mod cosmetic_attachment;
pub mod cosmetics;
pub mod error;
pub mod geometry;
pub mod math;
pub mod models;
pub mod output;
pub mod render_pipeline;
pub mod renderer;
pub mod scene;
pub mod skin;
pub mod texture;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::Error;
pub use error::Result;

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!("[INFO] {}", format!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!("[WARN] {}", format!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_print {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            print!($($arg)*);
        }
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!("[DEBUG] {}", format!($($arg)*));
        }
    }
}

/// High-level API for rendering a blockymodel to PNG
///
/// This function provides a convenient way to render a blockymodel file
/// to a PNG image, suitable for WASM integration.
pub fn render_blockymodel_to_png(
	model_path: &std::path::Path,
	texture_path: Option<&std::path::Path>,
	output_width: u32,
	output_height: u32,
) -> Result<image::RgbaImage> {
	let model = models::parse_blockymodel_from_file(model_path)?;

	let scene = scene::SceneGraph::from_blockymodel(&model)?;

	let texture = if let Some(tex_path) = texture_path {
		texture::Texture::from_file(tex_path)?
	} else {
		let texture_image =
			image::RgbaImage::from_pixel(256, 256, image::Rgba([255, 255, 255, 255]));
		texture::Texture::from_image(image::DynamicImage::ImageRgba8(texture_image))
	};

	let visible_shapes = scene.get_visible_shapes();
	let mut faces = Vec::new();
	for (node, transform) in &visible_shapes {
		if let Some(ref shape) = node.shape {
			let geometry = geometry::generate_geometry(shape, *transform);
			for face in geometry {
				faces.push(renderer::RenderableFace {
					face,
					transform: *transform,
					shape: Some(shape.clone()),
					node_name: None,
					texture: None,
					tint: None,
				});
			}
		}
	}

	let camera = camera::Camera::default_isometric();
	renderer::render_scene(&faces, &texture, &camera, output_width, output_height)
}
