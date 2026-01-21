//! Skin configuration parsing and tint gradient resolution
//!
//! Parses skin.json files and resolves tint gradient paths for different body parts.

use crate::error::{Error, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Root skin configuration from skin.json
#[derive(Debug, Clone, Deserialize)]
pub struct SkinConfig {
	pub skin: SkinData,
}

/// Skin customization data
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinData {
	/// Body/skin tone: "Default.10" -> Skin_Tones/10.png
	pub body_characteristic: String,
	/// Underwear style and color
	pub underwear: Option<String>,
	/// Face texture variant
	pub face: Option<String>,
	/// Ear style
	pub ears: Option<String>,
	/// Mouth style
	pub mouth: Option<String>,
	/// Haircut style and color: "WavyShort.BrownDark" -> Hair/Brown_Dark.png
	pub haircut: Option<String>,
	/// Facial hair style
	pub facial_hair: Option<String>,
	/// Eyebrow style and color: "Medium.BrownDark" -> Hair/Brown_Dark.png
	pub eyebrows: Option<String>,
	/// Eye style and color: "Plain_Eyes.Turquoise" -> Eyes/Turquoise.png
	pub eyes: Option<String>,
	/// Pants style and color
	pub pants: Option<String>,
	/// Overpants style
	pub overpants: Option<String>,
	/// Undertop style
	pub undertop: Option<String>,
	/// Overtop style
	pub overtop: Option<String>,
	/// Shoes style
	pub shoes: Option<String>,
	/// Head accessory
	pub head_accessory: Option<String>,
	/// Face accessory
	pub face_accessory: Option<String>,
	/// Ear accessory
	pub ear_accessory: Option<String>,
	/// Skin feature (tattoos, markings, etc.)
	pub skin_feature: Option<String>,
	/// Gloves style
	pub gloves: Option<String>,
	/// Cape style: "Cape_Royal_Emissary.Black.NoNeck" -> No Neck Variant of Cape Royal Emissary Black
	pub cape: Option<String>,
}

impl SkinConfig {
	/// Parse skin configuration from a JSON file
	pub fn from_file(path: &Path) -> Result<Self> {
		let content = std::fs::read_to_string(path).map_err(Error::Io)?;
		Self::from_str(&content)
	}

	/// Parse skin configuration from a JSON string
	pub fn from_str(json: &str) -> Result<Self> {
		serde_json::from_str(json).map_err(|e| Error::Parse(e.to_string()))
	}
}

/// Resolved tint gradients for rendering
#[derive(Debug, Clone)]
pub struct ResolvedTints {
	/// Base path for tint gradients (e.g., "assets/Common/TintGradients")
	pub base_path: PathBuf,
	/// Skin tone gradient path
	pub skin_tone: PathBuf,
	/// Eye color gradient path
	pub eye_color: Option<PathBuf>,
	/// Hair color gradient path (used for hair and eyebrows)
	pub hair_color: Option<PathBuf>,
	/// Underwear color gradient path (for pelvis area)
	pub underwear_color: Option<PathBuf>,
	/// Some Face Accessories are tintable, so we need to resolve the tint gradient path for them
	pub face_accessory_color: Option<PathBuf>,
	/// Cape color gradient path (or header color)
	pub cape_color: Option<PathBuf>,
	/// Gloves color gradient path
	pub gloves_color: Option<PathBuf>,
	/// Head accessory color gradient path
	pub head_accessory_color: Option<PathBuf>,
	/// Overpants color gradient path
	pub overpants_color: Option<PathBuf>,
	/// Overtop color gradient path
	pub overtop_color: Option<PathBuf>,
	/// Pants color gradient path
	pub pants_color: Option<PathBuf>,
	/// Shoes color gradient path
	pub shoes_color: Option<PathBuf>,
	/// Undertop color gradient path
	pub undertop_color: Option<PathBuf>,
}

impl ResolvedTints {
	/// Resolve tint gradient paths from skin configuration
	pub fn from_skin_config(
		config: &SkinConfig,
		base_path: &Path,
		registry: &crate::cosmetics::CosmeticRegistry,
	) -> Self {
		let skin_data = &config.skin;

		// Helper to resolve gradient set from registry
		let resolve_gradient_set = |cosmetic_id: Option<&String>,
		                            registry_map: &std::collections::HashMap<
			String,
			crate::cosmetics::CosmeticDefinition,
		>,
		                            default_set: Option<&str>|
		 -> Option<PathBuf> {
			let full_id = cosmetic_id?;
			// "WavyShort.BrownDark" -> "WavyShort"
			let id = full_id.split('.').next().unwrap_or(full_id);
			// "WavyShort.BrownDark" -> "BrownDark"
			// "Cape.Black.NoNeck" -> "Black" (second part)
			let parts: Vec<&str> = full_id.split('.').collect();
			let color_part = if parts.len() >= 2 {
				parts[1]
			} else {
				parts[0] // fallback or handle as "no color"? logic was just last() before
			};

			let def = registry_map.get(id)?;

			let gradient_set_id = def.gradient_set.as_deref().or(default_set)?;

			// If the cosmetic defines direct textures (like Face Accessory), we might not need a gradient set if it's not tintable
			// But if it IS tintable, it will have a gradient set.

			// Handle color name to file conversion
			// "BrownDark" -> "Brown_Dark"
			let color_file = camel_to_snake_case(color_part);

			// CHANGED: Check if the gradient set exists in the registry (handling "Fantasy_Cotton_Dark" -> "Dark_Fantasy_Cotton")
			// CHANGED: Check if the gradient set exists in the registry (handling "Fantasy_Cotton_Dark" -> "Dark_Fantasy_Cotton")
			if let Some(set) = registry.gradient_sets.get(gradient_set_id) {
				if let Some(grad_def) = set.gradients.get(color_part) {
					if let Some(texture_path) = &grad_def.texture {
						// Fix for path doubling: base_path is likely "assets/Common/TintGradients"
						// texture_path is likely "TintGradients/Folder/File.png"
						// We need to avoid "assets/Common/TintGradients/TintGradients/..."

						if base_path.ends_with("TintGradients")
							&& texture_path.starts_with("TintGradients")
						{
							if let Some(parent) = base_path.parent() {
								return Some(parent.join(texture_path));
							}
						}
						return Some(base_path.join(texture_path));
					}
				}
			}

			// Fallback to old behavior: assume gradient set ID is the folder name
			Some(
				base_path
					.join(gradient_set_id)
					.join(format!("{}.png", color_file)),
			)
		};

		// Parse bodyCharacteristic: "Default.10" -> "10"
		let skin_tone_num = skin_data
			.body_characteristic
			.split('.')
			.last()
			.unwrap_or("10");
		let skin_tone = base_path
			.join("Skin_Tones")
			.join(format!("{}.png", skin_tone_num));

		let eye_color = resolve_gradient_set(skin_data.eyes.as_ref(), &registry.eyes, Some("Eyes"));

		// Parse haircut: "WavyShort.BrownDark" -> "Brown_Dark" (with underscore)
		// Haircuts uses "Hair" gradient set typically
		let hair_color =
			resolve_gradient_set(skin_data.haircut.as_ref(), &registry.haircuts, Some("Hair"));

		// Parse underwear: "Bra.Blue" -> "Blue"
		// Underwear uses Colored_Cotton gradient set
		let underwear_color = resolve_gradient_set(
			skin_data.underwear.as_ref(),
			&registry.underwear,
			Some("Colored_Cotton"),
		);

		// Face Accessory
		let face_accessory_color = resolve_gradient_set(
			skin_data.face_accessory.as_ref(),
			&registry.face_accessories,
			None,
		);

		// Cape: "Cape_Royal.Black.NoNeck"
		// Capes typically use a gradient set from registry, or fall back to 'Colored_Cotton' or similar if not specified?
		// Actually many capes have "GradientSet", e.g. "Colored_Cotton".
		let cape_color = resolve_gradient_set(skin_data.cape.as_ref(), &registry.capes, None);

		// Gloves: "FlowerBracer.Gold_Red"
		let gloves_color = resolve_gradient_set(skin_data.gloves.as_ref(), &registry.gloves, None);

		// Head Accessory: "HeadphonesDadCap.Black"
		let head_accessory_color = resolve_gradient_set(
			skin_data.head_accessory.as_ref(),
			&registry.head_accessories,
			None,
		);

		// Overpants: "LongSocks_Torn.Torn"
		let overpants_color =
			resolve_gradient_set(skin_data.overpants.as_ref(), &registry.overpants, None);

		// Overtop: "GoldtrimJacket.Black"
		let overtop_color =
			resolve_gradient_set(skin_data.overtop.as_ref(), &registry.overtops, None);

		// Pants: "Colored_Trousers.Black"
		let pants_color = resolve_gradient_set(skin_data.pants.as_ref(), &registry.pants, None);

		// Shoes: "BasicBoots.Black"
		let shoes_color = resolve_gradient_set(skin_data.shoes.as_ref(), &registry.shoes, None);

		// Undertop: "PastelFade.Orange"
		let undertop_color =
			resolve_gradient_set(skin_data.undertop.as_ref(), &registry.undertops, None);

		ResolvedTints {
			base_path: base_path.to_path_buf(),
			skin_tone,
			eye_color,
			hair_color,
			underwear_color,
			face_accessory_color,
			cape_color,
			gloves_color,
			head_accessory_color,
			overpants_color,
			overtop_color,
			pants_color,
			shoes_color,
			undertop_color,
		}
	}
}

/// Convert camelCase color names to file format (e.g., "BrownDark" -> "Brown_Dark")
pub fn camel_to_snake_case(s: &str) -> String {
	let mut result = String::new();
	for (i, c) in s.chars().enumerate() {
		if c.is_uppercase() && i != 0 {
			result.push('_');
		}
		result.push(c);
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cosmetics::{CosmeticDefinition, CosmeticRegistry};
	use std::collections::HashMap;

	fn mock_registry() -> CosmeticRegistry {
		let mut registry = CosmeticRegistry {
			faces: HashMap::new(),
			eyes: HashMap::new(),
			eyebrows: HashMap::new(),
			mouths: HashMap::new(),
			ears: HashMap::new(),
			haircuts: HashMap::new(),
			facial_hair: HashMap::new(),
			underwear: HashMap::new(),
			face_accessories: HashMap::new(),
			capes: HashMap::new(),
			ear_accessories: HashMap::new(),
			gloves: HashMap::new(),
			head_accessories: HashMap::new(),
			gradient_sets: HashMap::new(),
			overpants: HashMap::new(),
			overtops: HashMap::new(),
			pants: HashMap::new(),
			shoes: HashMap::new(),
			undertops: HashMap::new(),
		};

		// Mock Haircut
		registry.haircuts.insert(
			"WavyShort".to_string(),
			CosmeticDefinition {
				id: "WavyShort".to_string(),
				gradient_set: Some("Hair".to_string()),
				hair_type: None,
				requires_generic_haircut: None,
				name: None,
				model: None,
				greyscale_texture: None,
				variants: None,
				textures: None,
				head_accessory_type: None,
				disable_character_part_category: None,
			},
		);

		// Mock Underwear
		registry.underwear.insert(
			"Bra".to_string(),
			CosmeticDefinition {
				id: "Bra".to_string(),
				gradient_set: Some("Colored_Cotton".to_string()),
				hair_type: None,
				requires_generic_haircut: None,
				name: None,
				model: None,
				greyscale_texture: None,
				variants: None,
				textures: None,
				head_accessory_type: None,
				disable_character_part_category: None,
			},
		);

		// Mock Face Accessory with custom gradient set
		registry.face_accessories.insert(
			"Goggles".to_string(),
			CosmeticDefinition {
				id: "Goggles".to_string(),
				gradient_set: Some("Faded_Leather".to_string()),
				hair_type: None,
				requires_generic_haircut: None,
				name: None,
				model: None,
				greyscale_texture: None,
				variants: None,
				textures: None,
				head_accessory_type: None,
				disable_character_part_category: None,
			},
		);

		// Mock Eyes
		registry.eyes.insert(
			"Plain_Eyes".to_string(),
			CosmeticDefinition {
				id: "Plain_Eyes".to_string(),
				gradient_set: Some("Eyes".to_string()),
				hair_type: None,
				requires_generic_haircut: None,
				name: None,
				model: None,
				greyscale_texture: None,
				variants: None,
				textures: None,
				head_accessory_type: None,
				disable_character_part_category: None,
			},
		);

		registry
	}

	#[test]
	fn test_resolve_tints_gradient_set_lookup() {
		let json = r#"{
            "skin": {
                "bodyCharacteristic": "Default.10",
                "cape": "Cape_Bannerlord.Red"
            }
        }"#;

		let config = SkinConfig::from_str(json).unwrap();
		let mut registry = mock_registry();

		// Add Cape_Bannerlord to registry
		registry.capes.insert(
			"Cape_Bannerlord".to_string(),
			CosmeticDefinition {
				id: "Cape_Bannerlord".to_string(),
				gradient_set: Some("Fantasy_Cotton_Dark".to_string()),
				hair_type: None,
				requires_generic_haircut: None,
				name: None,
				model: None,
				greyscale_texture: None,
				variants: None,
				textures: None,
				head_accessory_type: None,
				disable_character_part_category: None,
			},
		);

		// Add Gradient Set to registry
		let mut gradients = HashMap::new();
		gradients.insert(
			"Red".to_string(),
			crate::cosmetics::GradientDefinition {
				base_color: None,
				texture: Some("TintGradients/Dark_Fantasy_Cotton/Red.png".to_string()),
			},
		);

		registry.gradient_sets.insert(
			"Fantasy_Cotton_Dark".to_string(),
			crate::cosmetics::GradientSet {
				id: Some("Fantasy_Cotton_Dark".to_string()),
				gradients,
			},
		);

		let tints = ResolvedTints::from_skin_config(&config, Path::new("assets"), &registry);

		// Should resolve to the path specified in the gradient set, NOT "Fantasy_Cotton_Dark/Red.png"
		assert!(tints
			.cape_color
			.as_ref()
			.unwrap()
			.ends_with("TintGradients/Dark_Fantasy_Cotton/Red.png"));
	}

	#[test]
	fn test_parse_skin_json() {
		let json = r#"{
            "skin": {
                "bodyCharacteristic": "Default.10",
                "underwear": "Bra.Blue",
                "face": "Face_Neutral_Freckles",
                "ears": "Default",
                "mouth": "Mouth_Thin",
                "haircut": "WavyShort.BrownDark",
                "facialHair": null,
                "eyebrows": "Medium.BrownDark",
                "eyes": "Plain_Eyes.Turquoise",
                "pants": "Pants_Wasteland_Marauder.Orange",
                "overpants": "LongSocks_Torn.Torn",
                "undertop": "Voidbearer_CursedArm.Purple",
                "overtop": "Merchant_Tunic.Turquoise",
                "shoes": "Sneakers_Wasteland_Marauder.Brown",
                "headAccessory": "Battleworn_Helm.Green",
                "faceAccessory": null,
                "earAccessory": null,
                "skinFeature": null,
                "gloves": null,
                "cape": "Cape_Wasteland_Marauder.Red.NoNeck"
            }
        }"#;

		let config = SkinConfig::from_str(json).unwrap();
		assert_eq!(config.skin.body_characteristic, "Default.10");
		assert_eq!(config.skin.eyes, Some("Plain_Eyes.Turquoise".to_string()));
		assert_eq!(config.skin.haircut, Some("WavyShort.BrownDark".to_string()));
		assert_eq!(
			config.skin.cape,
			Some("Cape_Wasteland_Marauder.Red.NoNeck".to_string())
		);
	}

	#[test]
	fn test_resolve_tints() {
		let json = r#"{
            "skin": {
                "bodyCharacteristic": "Default.10",
                "underwear": "Bra.Blue",
                "eyes": "Plain_Eyes.Turquoise",
                "haircut": "WavyShort.BrownDark"
            }
        }"#;

		let config = SkinConfig::from_str(json).unwrap();
		let registry = mock_registry();
		let tints =
			ResolvedTints::from_skin_config(&config, Path::new("assets/TintGradients"), &registry);

		assert!(tints.skin_tone.ends_with("Skin_Tones/10.png"));
		assert!(tints
			.eye_color
			.as_ref()
			.unwrap()
			.ends_with("Eyes/Turquoise.png"));
		assert!(tints
			.hair_color
			.as_ref()
			.unwrap()
			.ends_with("Hair/Brown_Dark.png"));
		assert!(tints
			.underwear_color
			.as_ref()
			.unwrap()
			.ends_with("Colored_Cotton/Blue.png"));
	}

	#[test]
	fn test_resolve_tints_dynamic() {
		let json = r#"{
             "skin": {
                 "bodyCharacteristic": "Default.10",
                 "faceAccessory": "Goggles.Brown"
             }
         }"#;

		let config = SkinConfig::from_str(json).unwrap();
		let registry = mock_registry();
		let tints =
			ResolvedTints::from_skin_config(&config, Path::new("assets/TintGradients"), &registry);

		// Should use Faded_Leather from mock registry for Goggles
		assert!(tints
			.face_accessory_color
			.as_ref()
			.unwrap()
			.ends_with("Faded_Leather/Brown.png"));
	}

	#[test]
	fn test_camel_to_snake_case() {
		assert_eq!(camel_to_snake_case("BrownDark"), "Brown_Dark");
		assert_eq!(camel_to_snake_case("Brown"), "Brown");
		assert_eq!(camel_to_snake_case("LightBrown"), "Light_Brown");
		assert_eq!(camel_to_snake_case("ABC"), "A_B_C");
	}
}
