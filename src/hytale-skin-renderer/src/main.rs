//! CLI entry point for Hytale skin renderer

use hytale_skin_renderer::*;
use std::env;
use std::path::PathBuf;

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 2 {
		log_error!(
			"Usage: {} <blockymodel_path> [output_path] [texture_path]",
			args[0]
		);
		log_error!(
			"Example: {} assets/Common/Characters/Player.blockymodel output.png",
			args[0]
		);
		std::process::exit(1);
	}

	let model_path = PathBuf::from(&args[1]);
	let output_path = if args.len() > 2 {
		PathBuf::from(&args[2])
	} else {
		PathBuf::from("output.png")
	};

	let texture_path = if args.len() > 3 {
		Some(PathBuf::from(&args[3]))
	} else {
		None
	};

	log_info!("Loading blockymodel from: {:?}", model_path);

	let model = match models::parse_blockymodel_from_file(&model_path) {
		Ok(m) => m,
		Err(e) => {
			log_error!("Error parsing blockymodel: {}", e);
			std::process::exit(1);
		}
	};

	log_info!("Building scene graph...");
	let scene = match scene::SceneGraph::from_blockymodel(&model) {
		Ok(s) => s,
		Err(e) => {
			log_error!("Error building scene graph: {}", e);
			std::process::exit(1);
		}
	};

	let visible_shapes = scene.get_visible_shapes();
	log_info!("Found {} visible shapes", visible_shapes.len());

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
	log_info!("Generated {} faces", faces.len());

	let texture = if let Some(ref tex_path) = texture_path {
		log_info!("Loading texture from: {:?}", tex_path);
		match texture::Texture::from_file(tex_path) {
			Ok(t) => t,
			Err(e) => {
				log_error!("Error loading texture: {}", e);
				std::process::exit(1);
			}
		}
	} else {
		log_info!("Using default white texture");
		let texture_image =
			image::RgbaImage::from_pixel(256, 256, image::Rgba([255, 255, 255, 255]));
		texture::Texture::from_image(image::DynamicImage::ImageRgba8(texture_image))
	};

	let camera = camera::Camera::default_isometric();

	log_info!("Rendering to {}x{}...", 512, 512);
	let image = match renderer::render_scene(&faces, &texture, &camera, 512, 512) {
		Ok(img) => img,
		Err(e) => {
			log_error!("Error rendering scene: {}", e);
			std::process::exit(1);
		}
	};

	log_info!("Exporting to: {:?}", output_path);
	match output::export_png(&image, &output_path) {
		Ok(_) => log_info!("Successfully rendered to {:?}", output_path),
		Err(e) => {
			log_error!("Error exporting PNG: {}", e);
			std::process::exit(1);
		}
	}
}
