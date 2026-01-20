use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CosmeticDefinition {
    pub hair_type: Option<String>,
    pub requires_generic_haircut: Option<bool>,
    pub id: String,
    pub name: Option<String>,
    pub model: Option<String>,
    pub greyscale_texture: Option<String>,
    pub gradient_set: Option<String>,
    pub variants: Option<HashMap<String, CosmeticVariant>>,
    /// Direct textures map for cosmetics like EyePatch
    /// Key is the color name (e.g., "Black"), value contains texture path
    pub textures: Option<HashMap<String, TextureVariant>>,
    /// Head accessory type for hair culling: "Simple", "HalfCovering", "FullyCovering"
    pub head_accessory_type: Option<String>,
    /// Character part category to disable when this cosmetic is equipped (e.g., "Haircut")
    pub disable_character_part_category: Option<String>,
}

/// Texture variant with direct texture and base color
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextureVariant {
    pub texture: String,
    pub base_color: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CosmeticVariant {
    pub model: Option<String>,
    pub greyscale_texture: Option<String>,
    pub textures: Option<HashMap<String, TextureVariant>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GradientDefinition {
    pub base_color: Option<Vec<String>>,
    pub texture: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GradientSet {
    pub id: Option<String>,
    pub gradients: HashMap<String, GradientDefinition>,
}

#[derive(Debug, Clone)]
pub struct CosmeticRegistry {
    pub faces: HashMap<String, CosmeticDefinition>,
    pub eyes: HashMap<String, CosmeticDefinition>,
    pub eyebrows: HashMap<String, CosmeticDefinition>,
    pub mouths: HashMap<String, CosmeticDefinition>,
    pub ears: HashMap<String, CosmeticDefinition>,
    pub haircuts: HashMap<String, CosmeticDefinition>,
    pub facial_hair: HashMap<String, CosmeticDefinition>,
    pub underwear: HashMap<String, CosmeticDefinition>,
    pub face_accessories: HashMap<String, CosmeticDefinition>,
    pub capes: HashMap<String, CosmeticDefinition>,
    pub ear_accessories: HashMap<String, CosmeticDefinition>,
    pub gloves: HashMap<String, CosmeticDefinition>,
    pub head_accessories: HashMap<String, CosmeticDefinition>,
    pub gradient_sets: HashMap<String, GradientSet>,
    pub overpants: HashMap<String, CosmeticDefinition>,
    pub overtops: HashMap<String, CosmeticDefinition>,
    pub pants: HashMap<String, CosmeticDefinition>,
    pub shoes: HashMap<String, CosmeticDefinition>,
    pub undertops: HashMap<String, CosmeticDefinition>,
}

pub fn is_valid_cosmetic_id(id: &str) -> bool {
    !id.is_empty() && id != "null"
}

impl CosmeticRegistry {
    pub fn load_from_assets(assets_path: &Path) -> Result<Self> {
        let load_file = |path: &str| -> HashMap<String, CosmeticDefinition> {
            let mut map = HashMap::new();
            let full_path = assets_path.join(path);
            if let Ok(file) = File::open(&full_path) {
                let reader = BufReader::new(file);
                if let Ok(cosmetics) = serde_json::from_reader::<_, Vec<CosmeticDefinition>>(reader)
                {
                    for cosmetic in cosmetics {
                        map.insert(cosmetic.id.clone(), cosmetic);
                    }
                } else {
                    eprintln!("Failed to parse cosmetics from {}", path);
                }
            }
            map
        };

        let load_gradient_sets = |path: &str| -> HashMap<String, GradientSet> {
            let mut map = HashMap::new();
            let full_path = assets_path.join(path);
            if let Ok(file) = File::open(&full_path) {
                let reader = BufReader::new(file);
                if let Ok(sets) = serde_json::from_reader::<_, Vec<GradientSet>>(reader) {
                    for set in sets {
                        if let Some(id) = &set.id {
                            map.insert(id.clone(), set);
                        }
                    }
                } else {
                    eprintln!("Failed to parse gradient sets from {}", path);
                }
            }
            map
        };

        Ok(Self {
            faces: load_file("Cosmetics/CharacterCreator/Faces.json"),
            eyes: load_file("Cosmetics/CharacterCreator/Eyes.json"),
            eyebrows: load_file("Cosmetics/CharacterCreator/Eyebrows.json"),
            mouths: load_file("Cosmetics/CharacterCreator/Mouths.json"),
            ears: load_file("Cosmetics/CharacterCreator/Ears.json"),
            haircuts: load_file("Cosmetics/CharacterCreator/Haircuts.json"),
            facial_hair: load_file("Cosmetics/CharacterCreator/FacialHair.json"),
            underwear: load_file("Cosmetics/CharacterCreator/Underwear.json"),
            face_accessories: load_file("Cosmetics/CharacterCreator/FaceAccessory.json"),
            capes: load_file("Cosmetics/CharacterCreator/Capes.json"),
            ear_accessories: load_file("Cosmetics/CharacterCreator/EarAccessory.json"),
            gloves: load_file("Cosmetics/CharacterCreator/Gloves.json"),
            head_accessories: load_file("Cosmetics/CharacterCreator/HeadAccessory.json"),
            gradient_sets: load_gradient_sets("Cosmetics/CharacterCreator/GradientSets.json"),
            overpants: load_file("Cosmetics/CharacterCreator/Overpants.json"),
            overtops: load_file("Cosmetics/CharacterCreator/Overtops.json"),
            pants: load_file("Cosmetics/CharacterCreator/Pants.json"),
            shoes: load_file("Cosmetics/CharacterCreator/Shoes.json"),
            undertops: load_file("Cosmetics/CharacterCreator/Undertops.json"),
        })
    }

    pub fn get_haircut(&self, id: &str) -> Option<&CosmeticDefinition> {
        self.haircuts.get(id)
    }

    pub fn get_face_feature(&self, id: &str) -> Option<&CosmeticDefinition> {
        self.faces
            .get(id)
            .or_else(|| self.eyes.get(id))
            .or_else(|| self.eyebrows.get(id))
            .or_else(|| self.mouths.get(id))
            .or_else(|| self.ears.get(id))
    }
}
