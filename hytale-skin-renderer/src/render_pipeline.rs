use crate::{
    animation, camera,
    cosmetic_attachment::{self, TintedFace},
    cosmetics, models, renderer, scene, skin, texture,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum HeadAccessoryCulling {
    None,          // Simple accessories - no culling
    HalfCovering,  // Partial hair culling
    FullyCovering, // Strict hair culling
    DisableHair,   // Complete hair removal
}

pub struct BodyRenderer {
    pub scene: scene::SceneGraph,
    pub registry: Arc<cosmetics::CosmeticRegistry>,
    pub tint_config: renderer::TintConfig,
    pub faces: Vec<TintedFace>,
    pub shapes: Vec<models::Shape>,
    pub fallbacks: HashMap<String, String>,
    pub player_texture_dimensions: (u32, u32),
    pub active_head_accessory_culling: Option<HeadAccessoryCulling>,
    pub hair_face_range: Option<(usize, usize)>, // (start_index, end_index) of hair faces
}

impl BodyRenderer {
    pub fn new(
        model_path: &Path,
        anim_path: &Path,
        registry: Arc<cosmetics::CosmeticRegistry>,
        fallbacks_path: Option<&Path>,
        player_texture_dimensions: (u32, u32),
    ) -> crate::Result<Self> {
        let model = models::parse_blockymodel_from_file(model_path)
            .map_err(|e| crate::Error::Parse(e.to_string()))?;
        let animation = models::parse_blockyanim_from_file(anim_path)
            .map_err(|e| crate::Error::Parse(e.to_string()))?;

        let pose = animation::sample_animation(&animation, 0.0);
        let scene = scene::SceneGraph::from_blockymodel_with_pose(&model, &pose, None)?;

        // Load fallbacks
        let fallbacks = if let Some(path) = fallbacks_path {
            if let Ok(file) = std::fs::File::open(path) {
                let reader = std::io::BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Default tint config, will be updated by skin config
        let tint_config = renderer::TintConfig::default();

        Ok(Self {
            scene,
            registry,
            tint_config,
            faces: Vec::new(),
            shapes: Vec::new(),
            fallbacks,
            player_texture_dimensions,
            active_head_accessory_culling: None,
            hair_face_range: None,
        })
    }

    pub fn with_skin_config(
        mut self,
        skin_config_path: &Path,
        tint_base_path: &Path,
    ) -> crate::Result<Self> {
        let skin_config = skin::SkinConfig::from_file(skin_config_path)
            .map_err(|e| crate::Error::Parse(e.to_string()))?; // Simplification

        let tints =
            skin::ResolvedTints::from_skin_config(&skin_config, tint_base_path, &self.registry);

        let skin_tint = texture::TintGradient::from_file(&tints.skin_tone)?;
        self.tint_config = renderer::TintConfig::with_skin(skin_tint);
        self.tint_config.apply_resolved_tints(&tints);

        // Attach all cosmetics based on skin config
        self.attach_base_body();
        self.attach_from_skin_config(&skin_config);

        Ok(self)
    }

    fn attach_base_body(&mut self) {
        let node_names = [
            "Pelvis", "Belly", "Chest", "R-Thigh", "L-Thigh", "R-Arm", "L-Arm", "Head", "Neck",
        ];
        for name in node_names {
            if let Some(node) = cosmetic_attachment::find_node_by_name(&self.scene.nodes, name) {
                if name == "R-Thigh" || name == "L-Thigh" || name == "R-Arm" || name == "L-Arm" {
                    cosmetic_attachment::collect_all_shapes_from_node_tinted(
                        node,
                        &mut self.faces,
                        &mut self.shapes,
                    );
                } else {
                    cosmetic_attachment::add_single_shape_tinted(
                        node,
                        name,
                        &mut self.faces,
                        &mut self.shapes,
                    );
                }
            }
        }
    }

    fn attach_from_skin_config(&mut self, config: &skin::SkinConfig) {
        // Filter out Head front face when Face cosmetic is present
        if config.skin.face.is_some() {
            self.faces.retain(|render_face| {
                if let Some(name) = &render_face.node_name {
                    !(name == "Head" && render_face.face.texture_face == "front")
                } else {
                    true
                }
            });
        }

        if let Some(ref id) = config.skin.face {
            cosmetic_attachment::attach_cosmetic(
                id,
                &self.registry.faces,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }
        if let Some(ref fid) = config.skin.eyes {
            cosmetic_attachment::attach_cosmetic(
                fid,
                &self.registry.eyes,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }
        if let Some(ref fid) = config.skin.eyebrows {
            cosmetic_attachment::attach_cosmetic(
                fid,
                &self.registry.eyebrows,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }
        if let Some(ref id_full) = config.skin.mouth {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.mouths,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }
        if let Some(ref id_full) = config.skin.facial_hair {
            let cosmetic_id = id_full.split('.').next().unwrap();
            if cosmetics::is_valid_cosmetic_id(cosmetic_id) {
                cosmetic_attachment::attach_cosmetic(
                    id_full,
                    &self.registry.facial_hair,
                    &self.registry.gradient_sets,
                    &self.scene,
                    &mut self.faces,
                    &mut self.shapes,
                    &self.tint_config,
                );
            }
        }
        if let Some(ref id) = config.skin.ears {
            cosmetic_attachment::attach_cosmetic(
                id,
                &self.registry.ears,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Haircut logic
        if let Some(ref haircut_str) = config.skin.haircut {
            // Track face count before attaching hair
            let hair_start_index = self.faces.len();

            let mut parts = haircut_str.split('.');
            let haircut_id = parts.next().unwrap();
            let variant_or_color = parts.next();

            if let Some(def) = self.registry.haircuts.get(haircut_id) {
                // 1. Check for generic fallback
                if def.requires_generic_haircut.unwrap_or(false) {
                    if let Some(hair_type) = &def.hair_type {
                        if let Some(fallback_id) = self.fallbacks.get(hair_type) {
                            cosmetic_attachment::load_and_attach_cosmetic(
                                fallback_id,
                                &self.registry.haircuts,
                                &self.registry.gradient_sets,
                                &self.scene,
                                &mut self.faces,
                                &mut self.shapes,
                                &self.tint_config,
                            );
                        }
                    }
                }

                // 2. Attach main haircut or variant
                let mut attached = false;
                if let Some(v_id) = variant_or_color {
                    if let Some(variants) = &def.variants {
                        if variants.contains_key(v_id) {
                            cosmetic_attachment::attach_variant(
                                def,
                                v_id,
                                &self.registry.haircuts,
                                &self.registry.gradient_sets,
                                &self.scene,
                                &mut self.faces,
                                &mut self.shapes,
                                &self.tint_config,
                            );
                            attached = true;
                        }
                    }
                }

                if !attached {
                    cosmetic_attachment::load_and_attach_cosmetic(
                        haircut_id,
                        &self.registry.haircuts,
                        &self.registry.gradient_sets,
                        &self.scene,
                        &mut self.faces,
                        &mut self.shapes,
                        &self.tint_config,
                    );
                }
            }

            // Record hair face range for later culling
            let hair_end_index = self.faces.len();
            if hair_end_index > hair_start_index {
                self.hair_face_range = Some((hair_start_index, hair_end_index));
            }
        }

        // Underwear
        if let Some(ref id) = config.skin.underwear {
            let type_id = id.split('.').next().unwrap();
            cosmetic_attachment::attach_cosmetic(
                type_id,
                &self.registry.underwear,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Face Accessory
        if let Some(ref id_full) = config.skin.face_accessory {
            cosmetic_attachment::attach_face_accessory(
                id_full,
                &self.registry.face_accessories,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Cape
        if let Some(ref id_full) = config.skin.cape {
            cosmetic_attachment::attach_cape(
                id_full,
                &self.registry.capes,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Ear Accessory
        if let Some(ref id_full) = config.skin.ear_accessory {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.ear_accessories,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Gloves
        if let Some(ref id_full) = config.skin.gloves {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.gloves,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Head Accessory
        if let Some(ref id_full) = config.skin.head_accessory {
            let cosmetic_id = id_full.split('.').next().unwrap();
            if let Some(def) = self.registry.head_accessories.get(cosmetic_id) {
                // Determine culling mode from accessory definition
                self.active_head_accessory_culling = Some(
                    if def.disable_character_part_category.as_deref() == Some("Haircut") {
                        HeadAccessoryCulling::DisableHair
                    } else if def.head_accessory_type.as_deref() == Some("FullyCovering") {
                        HeadAccessoryCulling::FullyCovering
                    } else if def.head_accessory_type.as_deref() == Some("HalfCovering") {
                        HeadAccessoryCulling::HalfCovering
                    } else {
                        HeadAccessoryCulling::None
                    },
                );
            }

            // Track face count before attaching to identify head accessory faces
            let face_count_before = self.faces.len();

            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.head_accessories,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );

            // Dynamic spatial culling: Identify and remove faces that are internal to the head volume.
            // This preserves external faces (like medallions hanging below the head) while removing
            // the bottom caps of hats/bandanas that are inside the head.
            let head_node = cosmetic_attachment::find_node_by_name(&self.scene.nodes, "Head");
            let head_info = head_node.and_then(|node| {
                node.shape.as_ref().map(|shape| {
                    let size = shape.settings.size.unwrap_or(models::Vector3::zero());
                    let half_x = (size.x / 2.0) * shape.stretch.x;
                    let half_y = (size.y / 2.0) * shape.stretch.y;
                    let half_z = (size.z / 2.0) * shape.stretch.z;

                    let min_x = shape.offset.x - half_x;
                    let max_x = shape.offset.x + half_x;
                    let min_y = shape.offset.y - half_y;
                    let max_y = shape.offset.y + half_y;
                    let min_z = shape.offset.z - half_z;
                    let max_z = shape.offset.z + half_z;

                    (
                        min_x,
                        max_x,
                        min_y,
                        max_y,
                        min_z,
                        max_z,
                        node.transform.inverse(),
                    )
                })
            });

            let mut i = face_count_before;
            while i < self.faces.len() {
                let face_type = &self.faces[i].face.texture_face;
                let node_name = &self.faces[i].node_name;

                let mut should_remove = false;

                if let Some((min_x, max_x, min_y, max_y, min_z, max_z, head_inv_transform)) =
                    head_info
                {
                    // Calculate face center in world space
                    let mut world_center = glam::Vec3::ZERO;
                    for v in &self.faces[i].face.vertices {
                        world_center += v.position;
                    }
                    world_center /= self.faces[i].face.vertices.len() as f32;

                    // Transform center to Head local space
                    let local_center = head_inv_transform.transform_point3(world_center);

                    // A face is considered "internal" if it is within the head's volume
                    let is_spatially_internal = local_center.x > min_x - 0.1
                        && local_center.x < max_x + 0.1
                        && local_center.y > min_y - 0.1
                        && local_center.y < max_y + 0.1
                        && local_center.z > min_z - 0.1
                        && local_center.z < max_z + 0.1;

                    if face_type == "bottom" && is_spatially_internal {
                        should_remove = true;
                    } else if face_type == "top"
                        && is_spatially_internal
                        && node_name.as_ref().is_some_and(|n| n.contains("Base"))
                    {
                        should_remove = true;
                    }
                } else if face_type == "bottom" {
                    // Fallback to old logic if Head node not found
                    should_remove = true;
                }

                if should_remove {
                    self.faces.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        // Overpants
        if let Some(ref id_full) = config.skin.overpants {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.overpants,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Overtop
        if let Some(ref id_full) = config.skin.overtop {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.overtops,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Pants
        if let Some(ref id_full) = config.skin.pants {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.pants,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Shoes
        if let Some(ref id_full) = config.skin.shoes {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.shoes,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Undertop
        if let Some(ref id_full) = config.skin.undertop {
            cosmetic_attachment::attach_cosmetic(
                id_full,
                &self.registry.undertops,
                &self.registry.gradient_sets,
                &self.scene,
                &mut self.faces,
                &mut self.shapes,
                &self.tint_config,
            );
        }

        // Apply hair culling based on head accessory (must be done AFTER head accessory is attached)
        if let Some(ref culling_mode) = self.active_head_accessory_culling {
            if let Some((hair_start, hair_end)) = self.hair_face_range {
                match culling_mode {
                    HeadAccessoryCulling::DisableHair => {
                        // Remove ALL hair faces in the tracked range
                        self.faces.drain(hair_start..hair_end);
                    }
                    HeadAccessoryCulling::FullyCovering | HeadAccessoryCulling::HalfCovering => {
                        // Apply part-based culling only to hair faces
                        cosmetic_attachment::apply_hair_culling_to_range(
                            &mut self.faces,
                            hair_start,
                            hair_end,
                            culling_mode,
                        );
                    }
                    HeadAccessoryCulling::None => {
                        // No culling needed
                    }
                }
            }
        }
    }

    pub fn render(
        &self,
        camera: &dyn camera::CameraProjection,
        output_width: u32,
        output_height: u32,
        base_texture_path: &Path,
    ) -> crate::Result<image::RgbaImage> {
        let texture = texture::Texture::from_file(base_texture_path)?;

        renderer::render_scene_tinted(
            &self.faces,
            &texture,
            camera,
            output_width,
            output_height,
            &self.tint_config,
        )
    }
}
