use crate::{cosmetics, geometry, models, renderer, scene, texture};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;


/// Extended face that includes node name for tint mapping and specific texture/tint
pub type TintedFace = renderer::RenderableFace;

pub fn find_node_by_name<'a>(
	nodes: &'a [scene::SceneNode],
	name: &str,
) -> Option<&'a scene::SceneNode> {
	for node in nodes {
		if node.name == name {
			return Some(node);
		}
		if let Some(found) = find_node_by_name(&node.children, name) {
			return Some(found);
		}
	}
	None
}

pub fn collect_all_shapes_from_node_tinted(
	node: &scene::SceneNode,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
) {
	// Filter out default lids if we are applying cosmetics (heuristic: cosmetics usually replace these)
	// let skip_nodes = ["L-Eyelid", "R-Eyelid", "L-Eyelid-Bot", "R-Eyelid-Bot"];
	// if skip_nodes.contains(&node.name.as_str()) {
	// 	return;
	// }

	if let Some(ref shape) = node.shape {
		if shape.visible {
			let geometry = geometry::generate_geometry(shape, node.transform);

			for face in geometry {
				faces.push(renderer::RenderableFace {
					face,
					transform: node.transform,
					shape: Some(shape.clone()),
					node_name: Some(node.name.clone()),
					texture: None,
					tint: None,
				});
			}
			shapes.push(shape.clone());
		}
	}

	// Recursively collect from children
	for child in &node.children {
		collect_all_shapes_from_node_tinted(child, faces, shapes);
	}
}

pub fn add_single_shape_tinted(
	node: &scene::SceneNode,
	name: &str,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
) {
	if let Some(ref shape) = node.shape {
		if shape.visible {
			let geometry = geometry::generate_geometry(shape, node.transform);
			for face in geometry {
				faces.push(renderer::RenderableFace {
					face,
					transform: node.transform,
					shape: Some(shape.clone()),
					node_name: Some(name.to_string()),
					texture: None,
					tint: None,
				});
			}
			shapes.push(shape.clone());
		}
	}
}

pub fn load_and_attach_cosmetic(
	cosmetic_id: &str,
	registry: &HashMap<String, cosmetics::CosmeticDefinition>,
	_gradient_sets: &HashMap<String, cosmetics::GradientSet>,
	scene: &scene::SceneGraph,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	tint_config: &renderer::TintConfig,
) {
	if let Some(def) = registry.get(cosmetic_id) {
		let model_path_str = match &def.model {
			Some(m) => m,
			None => return,
		};
		let texture_path_str = match &def.greyscale_texture {
			Some(t) => t,
			None => return,
		};

		let model_path = Path::new("assets/Common").join(model_path_str);

		if let Ok(model) = models::parse_blockymodel_from_file(&model_path) {
			let texture_path = Path::new("assets/Common").join(texture_path_str);
			let texture = texture::Texture::from_file(&texture_path)
				.ok()
				.map(Arc::new);

			let tint = match def.gradient_set.as_deref() {
				Some("Skin") => Some(Arc::new(tint_config.skin.clone())),
				Some("Hair") => tint_config.hair.as_ref().map(|t| Arc::new(t.clone())),
				Some("Eyes_Gradient") => tint_config.eyes.as_ref().map(|t| Arc::new(t.clone())),
				Some(_set_id) => {
					// Try to find a default tint for this set?
					// Usually this function (`load_and_attach_cosmetic`) is for simple attachments without color params.
					// So we probably don't have a specific color to look up.
					// But if it needs a tint from a set, we might default to something?
					// For now, let's keep the None behavior unless we want to default to "Black" or similar.
					None
				}
				_ => None,
			};

			for root_node in &model.nodes {
				if let Some(anchor_node) = find_node_by_name(&scene.nodes, &root_node.name) {
					let anchor_offset = if let Some(ref s) = anchor_node.shape {
						glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
					} else {
						glam::Vec3::ZERO
					};
					let initial_parent_transform =
						anchor_node.transform * glam::Mat4::from_translation(anchor_offset);

					process_children(
						&root_node.children,
						initial_parent_transform,
						faces,
						shapes,
						&texture,
						&tint,
						tint_config,
						true,                    // check_tint_config
						def.id.contains("Face"), // is_face
						scene,
					);
				}
			}
		} else {
			eprintln!("  Failed to load model: {:?}", model_path);
		}
	} else {
		eprintln!("Cosmetic ID {} not found in registry", cosmetic_id);
	}
}

pub fn attach_variant(
	def: &cosmetics::CosmeticDefinition,
	variant_id: &str,
	_registry: &HashMap<String, cosmetics::CosmeticDefinition>,
	gradient_sets: &HashMap<String, cosmetics::GradientSet>,
	scene: &scene::SceneGraph,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	tint_config: &renderer::TintConfig,
) {
	if let Some(variants) = &def.variants {
		if let Some(variant) = variants.get(variant_id) {
			let mut variant_def = def.clone();
			variant_def.model = variant.model.clone();
			variant_def.greyscale_texture = variant.greyscale_texture.clone();

			let vid = variant_def.id.clone();
			let mut temp_registry = HashMap::new();
			temp_registry.insert(vid.clone(), variant_def);

			load_and_attach_cosmetic(
				&vid,
				&temp_registry,
				gradient_sets,
				scene,
				faces,
				shapes,
				tint_config,
			);
		}
	}
}

fn process_children(
	children: &[models::Node],
	parent_transform: glam::Mat4,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	texture: &Option<Arc<texture::Texture>>,
	tint: &Option<Arc<texture::TintGradient>>,
	tint_config: &renderer::TintConfig,
	check_tint_config: bool,
	is_face: bool,
	scene: &scene::SceneGraph,
) {
	for child in children {
		// Check if this node should snap to a player bone.
		let is_piece = child
			.shape
			.as_ref()
			.and_then(|s| s.settings.is_piece)
			.unwrap_or(false);

		let has_visible_shape = child
			.shape
			.as_ref()
			.map(|s| s.visible && s.shape_type != models::ShapeType::None)
			.unwrap_or(false);

		// Find matching player bone
		let player_bone = find_node_by_name(&scene.nodes, &child.name);

		// Snap if: isPiece flag, OR (matches player bone AND no visible shape)
		let should_snap = is_piece || (player_bone.is_some() && !has_visible_shape);

		let world_transform = if should_snap {
			if let Some(anchor) = player_bone {
				// Use the player bone's full transform (including rotation!)
				anchor.transform
			} else {
				// Fallback if no matching bone found
				let local_pos = crate::math::vec3_from_blockymodel(child.position);
				let local_rot = crate::math::quat_from_blockymodel(child.orientation);
				let local_transform = glam::Mat4::from_rotation_translation(local_rot, local_pos);
				parent_transform * local_transform
			}
		} else {
			let local_pos = crate::math::vec3_from_blockymodel(child.position);
			let local_rot = crate::math::quat_from_blockymodel(child.orientation);
			let local_transform = glam::Mat4::from_rotation_translation(local_rot, local_pos);
			parent_transform * local_transform
		};

		if let Some(ref shape) = child.shape {
			if shape.visible {
				// Use get_tint_for_node to check if this node should be tinted
				// (it returns None for parts that shouldn't be tinted, like eye backgrounds/sclera)
				let active_tint = if check_tint_config {
					if tint_config.get_tint_for_node(&child.name).is_some() {
						tint.clone()
					} else {
						None
					}
				} else {
					tint.clone()
				};

				let geometry = geometry::generate_geometry(shape, world_transform);

				for face in geometry {
					faces.push(renderer::RenderableFace {
						face,
						transform: world_transform,
						shape: Some(shape.clone()),
						node_name: Some(child.name.clone()),
						texture: texture.clone(),
						tint: active_tint.clone(),
					});
				}
				shapes.push(shape.clone());
			}
		}

		// For children of snapping nodes, use the matching player bone's offset
		let child_parent_transform = if should_snap {
			if let Some(anchor) = player_bone {
				let anchor_offset = if let Some(ref s) = anchor.shape {
					glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
				} else {
					glam::Vec3::ZERO
				};
				world_transform * glam::Mat4::from_translation(anchor_offset)
			} else {
				let child_offset = if let Some(ref s) = child.shape {
					glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
				} else {
					glam::Vec3::ZERO
				};
				world_transform * glam::Mat4::from_translation(child_offset)
			}
		} else {
			let child_offset = if let Some(ref s) = child.shape {
				glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
			} else {
				glam::Vec3::ZERO
			};
			world_transform * glam::Mat4::from_translation(child_offset)
		};

		process_children(
			&child.children,
			child_parent_transform,
			faces,
			shapes,
			texture,
			tint,
			tint_config,
			check_tint_config,
			is_face,
			scene,
		);
	}
}

pub fn attach_cosmetic(
	id_full: &str,
	registry: &HashMap<String, cosmetics::CosmeticDefinition>,
	gradient_sets: &HashMap<String, cosmetics::GradientSet>,
	scene: &scene::SceneGraph,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	tint_config: &renderer::TintConfig,
) {
	let parts: Vec<&str> = id_full.split('.').collect();
	let cosmetic_id = parts[0];

	// Heuristic for variant vs color:
	// We check against the registry definition to see what matches.
	// Common patterns:
	// - ID.Color (FaceAccessory)
	// - ID.Color.Variant (Cape)
	// - ID.Variant (EarAccessory sometimes?)
	// We will collect "modifiers" from the parts.
	let modifiers = parts.iter().skip(1).copied().collect::<Vec<&str>>();

	if let Some(def) = registry.get(cosmetic_id) {
		// 1. Resolve Variant
		// Find if any modifier matches a variant key.
		let variant_id = def.variants.as_ref().and_then(|variants| {
			modifiers
				.iter()
				.find(|&&m| variants.contains_key(m))
				.copied()
		});

		// 2. Resolve Color
		// Find if any modifier looks like a color.
		// For Capes: ID.Color.Variant -> Color is modifiers[0] if variant is modifiers[1].
		let color_id = modifiers.iter().find(|&&m| Some(m) != variant_id).copied();

		// 3. Determine Model and Texture based on selection
		let (model_path_opt, texture_path_opt, texture_base_colors) =
			resolve_model_and_texture(def, variant_id, color_id);

		if let Some(model_path_str) = model_path_opt {
			let model_path = Path::new("assets/Common").join(model_path_str);
			if let Ok(model) = models::parse_blockymodel_from_file(&model_path) {
				// 4. Load Texture
				let texture = if let Some(tex_path_str) = texture_path_opt {
					let tex_path = Path::new("assets/Common").join(tex_path_str);
					match texture::Texture::from_file(&tex_path) {
						Ok(tex) => Some(Arc::new(tex)),
						Err(e) => {
							eprintln!("  Failed to load cosmetic texture: {:?} - {}", tex_path, e);
							None
						}
					}
				} else {
					None
				};

				let tint = if let Some(_colors) = texture_base_colors {
					None
				} else {
					// Check Gradient Set
					match def.gradient_set.as_deref() {
						Some("Skin") => Some(Arc::new(tint_config.skin.clone())),
						Some("Hair") => tint_config.hair.as_ref().map(|t| Arc::new(t.clone())),
						Some("Eyes_Gradient") => {
							tint_config.eyes.as_ref().map(|t| Arc::new(t.clone()))
						}
						Some(other_gradient) => {
							// Try to load dynamic gradient if color is known
							if let Some(color) = color_id {
								// CHANGED: Use registry lookup first
								if let Some(set) = gradient_sets.get(other_gradient) {
									if let Some(grad_def) = set.gradients.get(color) {
										if let Some(texture_path_str) = &grad_def.texture {
											let gradient_path = if texture_path_str
												.starts_with("TintGradients")
											{
												Path::new("assets/Common").join(texture_path_str)
											} else {
												Path::new("assets/Common/TintGradients")
													.join(texture_path_str)
											};

											let gradient =
												texture::TintGradient::from_file(&gradient_path)
													.ok()
													.map(Arc::new);
											gradient
										} else {
											None
										}
									} else {
										None
									}
								} else {
									// Fallback to old behavior
									let gradient_path = Path::new("assets/Common/TintGradients")
										.join(other_gradient)
										.join(format!("{}.png", color));
									texture::TintGradient::from_file(&gradient_path)
										.ok()
										.map(Arc::new)
								}
							} else {
								// Fallback: try "Black" or similar if needed, or just None
								None
							}
						}
						None => None,
					}
				};

				// 6. Attach
				for root_node in &model.nodes {
					if let Some(anchor_node) = find_node_by_name(&scene.nodes, &root_node.name) {
						let anchor_offset = if let Some(ref s) = anchor_node.shape {
							glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
						} else {
							glam::Vec3::ZERO
						};
						let initial_parent_transform =
							anchor_node.transform * glam::Mat4::from_translation(anchor_offset);

						process_children(
							&root_node.children,
							initial_parent_transform,
							faces,
							shapes,
							&texture,
							&tint,
							tint_config,
							false, // check_tint_config (false for attachments)
							false, // is_face (attachments are not face parts usually)
							scene,
						);
					}
				}
			} else {
				eprintln!("  Failed to load cosmetic model: {:?}", model_path);
			}
		}
	}
}

fn resolve_model_and_texture(
	def: &cosmetics::CosmeticDefinition,
	variant_id: Option<&str>,
	color_id: Option<&str>,
) -> (Option<String>, Option<String>, Option<Vec<String>>) {
	// 1. Determine active variant definition (or base)
	let (model, textures, greyscale) = if let Some(vid) = variant_id {
		if let Some(variants) = &def.variants {
			if let Some(v) = variants.get(vid) {
				// Variant-specific overrides
				// If variant model is None, fallback to def model (common?) - No, usually variant has model.
				// But structure allows partial overrides?
				// CosmeticVariant struct: model: Option, greyscale: Option, textures: Option

				let m = v.model.clone().or(def.model.clone());
				let t_map = v.textures.clone().or(def.textures.clone());
				let g_tex = v
					.greyscale_texture
					.clone()
					.or(def.greyscale_texture.clone());
				(m, t_map, g_tex)
			} else {
				(
					def.model.clone(),
					def.textures.clone(),
					def.greyscale_texture.clone(),
				)
			}
		} else {
			(
				def.model.clone(),
				def.textures.clone(),
				def.greyscale_texture.clone(),
			)
		}
	} else {
		(
			def.model.clone(),
			def.textures.clone(),
			def.greyscale_texture.clone(),
		)
	};

	// 2. Determine Texture
	// Priority: Specific Texture for Color > Greyscale Texture
	let (final_texture, base_colors) = if let Some(c_id) = color_id {
		if let Some(map) = &textures {
			if let Some(tex_var) = map.get(c_id) {
				(Some(tex_var.texture.clone()), tex_var.base_color.clone())
			} else {
				// Color not in direct texture map, maybe it's for the gradient?
				(greyscale, None)
			}
		} else {
			(greyscale, None)
		}
	} else {
		// No color specified.
		// If we have a Textures map, we might Default to first?
		if let Some(map) = &textures {
			// Try "Black" or first
			if let Some(tex_var) = map.get("Black").or_else(|| map.values().next()) {
				(Some(tex_var.texture.clone()), tex_var.base_color.clone())
			} else {
				(greyscale, None)
			}
		} else {
			(greyscale, None)
		}
	};

	(model, final_texture, base_colors)
}

// Deprecated/Wrapper functions for backward compatibility or ease of use
pub fn attach_face_accessory(
	id_full: &str,
	registry: &HashMap<String, cosmetics::CosmeticDefinition>,
	gradient_sets: &HashMap<String, cosmetics::GradientSet>,
	scene: &scene::SceneGraph,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	tint_config: &renderer::TintConfig,
) {
	attach_cosmetic(
		id_full,
		registry,
		gradient_sets,
		scene,
		faces,
		shapes,
		tint_config,
	);
}

pub fn attach_cape(
	id_full: &str,
	registry: &HashMap<String, cosmetics::CosmeticDefinition>,
	gradient_sets: &HashMap<String, cosmetics::GradientSet>,
	scene: &scene::SceneGraph,
	faces: &mut Vec<TintedFace>,
	shapes: &mut Vec<models::Shape>,
	tint_config: &renderer::TintConfig,
) {
	attach_cosmetic(
		id_full,
		registry,
		gradient_sets,
		scene,
		faces,
		shapes,
		tint_config,
	);
}

/// Check if a node name belongs to a hair cosmetic
/// This includes both the main hair nodes (HairBase, Bangs, etc.) and their common child parts
pub fn is_hair_node(node_name: &str) -> bool {
	// Exclude non-hair body parts that might have similar names
	let exclusions = [
		"Eye", "Arm", "Leg", "Pelvis", "Chest", "Belly", "Thigh", "Neck",
	];
	if exclusions.iter().any(|pattern| node_name.contains(pattern)) {
		return false;
	}

	// Main hair identifiers
	let hair_patterns = ["Hair", "hair", "Bangs", "bangs", "Bun", "Puff"];
	if hair_patterns
		.iter()
		.any(|pattern| node_name.contains(pattern))
	{
		return true;
	}

	// Common hair part names (children of HairBase, etc.)
	// These are generic but commonly used in hair models
	let hair_part_patterns = [
		"Top", "Side", "Front", "Back", "Strand", "Corner", "Long", "Afro", "Curl", "Wave",
	];
	hair_part_patterns
		.iter()
		.any(|pattern| node_name.contains(pattern))
}

/// Apply part-based culling to hair faces in a specific range
pub fn apply_hair_culling_to_range(
	faces: &mut Vec<TintedFace>,
	start_index: usize,
	end_index: usize,
	culling_mode: &crate::render_pipeline::HeadAccessoryCulling,
) {
	use crate::render_pipeline::HeadAccessoryCulling;

	// Define which parts to keep based on culling mode
	let should_keep_part = |node_name: &str| -> bool {
		match culling_mode {
			HeadAccessoryCulling::FullyCovering => {
				// Keep ONLY HairBase for fully covering accessories
				node_name.contains("HairBase") || node_name == "Base"
			}
			HeadAccessoryCulling::HalfCovering => {
				// Keep HairBase (wraps the head)
				if node_name.contains("HairBase") || node_name == "Base" {
					return true;
				}
				// Remove top parts but keep sides, back, and front strands
				let top_parts = ["Top"];
				!top_parts.iter().any(|&part| node_name.contains(part))
			}
			_ => true, // Keep all parts for other modes
		}
	};

	// Remove faces in the range that should be culled (iterate backwards to avoid index issues)
	let mut i = end_index;
	while i > start_index {
		i -= 1;
		if let Some(name) = &faces[i].node_name {
			if !should_keep_part(name) {
				faces.remove(i);
			}
		}
	}
}
