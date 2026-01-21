use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::asset_provider::AssetProvider;

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

	pub fn load_from_json_assets(
		faces_json: &str,
		eyes_json: &str,
		eyebrows_json: &str,
		mouths_json: &str,
		ears_json: &str,
		haircuts_json: &str,
		facial_hair_json: &str,
		underwear_json: &str,
		face_accessories_json: &str,
		capes_json: &str,
		ear_accessories_json: &str,
		gloves_json: &str,
		head_accessories_json: &str,
		gradient_sets_json: &str,
		overpants_json: &str,
		overtops_json: &str,
		pants_json: &str,
		shoes_json: &str,
		undertops_json: &str,
	) -> Result<Self> {
		let load_json_file = |json: &str| -> Result<HashMap<String, CosmeticDefinition>> {
			let mut map = HashMap::new();
			let cosmetics: Vec<CosmeticDefinition> = serde_json::from_str(json)?;
			for cosmetic in cosmetics {
				map.insert(cosmetic.id.clone(), cosmetic);
			}
			Ok(map)
		};

		let load_json_gradient_sets = |json: &str| -> Result<HashMap<String, GradientSet>> {
			let mut map = HashMap::new();
			let sets: Vec<GradientSet> = serde_json::from_str(json)?;
			for set in sets {
				if let Some(id) = &set.id {
					map.insert(id.clone(), set);
				}
			}
			Ok(map)
		};

		Ok(Self {
			faces: load_json_file(faces_json)?,
			eyes: load_json_file(eyes_json)?,
			eyebrows: load_json_file(eyebrows_json)?,
			mouths: load_json_file(mouths_json)?,
			ears: load_json_file(ears_json)?,
			haircuts: load_json_file(haircuts_json)?,
			facial_hair: load_json_file(facial_hair_json)?,
			underwear: load_json_file(underwear_json)?,
			face_accessories: load_json_file(face_accessories_json)?,
			capes: load_json_file(capes_json)?,
			ear_accessories: load_json_file(ear_accessories_json)?,
			gloves: load_json_file(gloves_json)?,
			head_accessories: load_json_file(head_accessories_json)?,
			gradient_sets: load_json_gradient_sets(gradient_sets_json)?,
			overpants: load_json_file(overpants_json)?,
			overtops: load_json_file(overtops_json)?,
			pants: load_json_file(pants_json)?,
			shoes: load_json_file(shoes_json)?,
			undertops: load_json_file(undertops_json)?,
		})
	}

	pub fn load_from_provider(
		asset_provider: &dyn AssetProvider,
		asset_root: &str,
	) -> Result<Self> {
		let load_json = |path: &str| -> Result<String> {
			let full_path = format!("{}/{}", asset_root, path);
			let bytes = asset_provider.load_bytes(&full_path)?;
			String::from_utf8(bytes).map_err(|e| crate::Error::InvalidData(e.to_string()))
		};

		Self::load_from_json_assets(
			&load_json("Cosmetics/CharacterCreator/Faces.json")?,
			&load_json("Cosmetics/CharacterCreator/Eyes.json")?,
			&load_json("Cosmetics/CharacterCreator/Eyebrows.json")?,
			&load_json("Cosmetics/CharacterCreator/Mouths.json")?,
			&load_json("Cosmetics/CharacterCreator/Ears.json")?,
			&load_json("Cosmetics/CharacterCreator/Haircuts.json")?,
			&load_json("Cosmetics/CharacterCreator/FacialHair.json")?,
			&load_json("Cosmetics/CharacterCreator/Underwear.json")?,
			&load_json("Cosmetics/CharacterCreator/FaceAccessory.json")?,
			&load_json("Cosmetics/CharacterCreator/Capes.json")?,
			&load_json("Cosmetics/CharacterCreator/EarAccessory.json")?,
			&load_json("Cosmetics/CharacterCreator/Gloves.json")?,
			&load_json("Cosmetics/CharacterCreator/HeadAccessory.json")?,
			&load_json("Cosmetics/CharacterCreator/GradientSets.json")?,
			&load_json("Cosmetics/CharacterCreator/Overpants.json")?,
			&load_json("Cosmetics/CharacterCreator/Overtops.json")?,
			&load_json("Cosmetics/CharacterCreator/Pants.json")?,
			&load_json("Cosmetics/CharacterCreator/Shoes.json")?,
			&load_json("Cosmetics/CharacterCreator/Undertops.json")?,
		)
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
