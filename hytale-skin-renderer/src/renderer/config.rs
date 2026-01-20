use crate::skin::ResolvedTints;
use crate::texture::TintGradient;
use glam::Vec3;
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct TintConfig {
    /// Tint for skin (body, head, hands, feet)
    pub skin: TintGradient,
    /// Tint for eyes (optional)
    pub eyes: Option<TintGradient>,
    /// Tint for hair and eyebrows (optional)
    pub hair: Option<TintGradient>,
    /// Tint for underwear (used for thigh blending) - brightness is automatically applied
    pub underwear: Option<TintGradient>,
    /// Tint for cape
    pub cape: Option<TintGradient>,
    /// Tint for gloves
    pub gloves: Option<TintGradient>,
    /// Tint for head accessories
    pub head_accessories: Option<TintGradient>,
    /// Overpants
    pub overpants: Option<TintGradient>,
    /// Overtop
    pub overtop: Option<TintGradient>,
    /// Pants
    pub pants: Option<TintGradient>,
    /// Shoes
    pub shoes: Option<TintGradient>,
    /// Undertop
    pub undertop: Option<TintGradient>,
}

impl Default for TintConfig {
    fn default() -> Self {
        TintConfig {
            skin: TintGradient::identity(),
            eyes: None,
            hair: None,
            underwear: None,
            cape: None,
            gloves: None,
            head_accessories: None,
            overpants: None,
            overtop: None,
            pants: None,
            shoes: None,
            undertop: None,
        }
    }
}

impl TintConfig {
    /// Create a tint config with just skin tint
    pub fn with_skin(skin: TintGradient) -> Self {
        TintConfig {
            skin,
            eyes: None,
            hair: None,
            underwear: None,
            cape: None,
            gloves: None,
            head_accessories: None,
            overpants: None,
            overtop: None,
            pants: None,
            shoes: None,
            undertop: None,
        }
    }

    /// Get the appropriate tint gradient for a body part based on its node name
    /// Returns None for parts that should not be tinted (e.g., eye backgrounds/sclera)
    pub fn get_tint_for_node(&self, node_name: &str) -> Option<&TintGradient> {
        // Eye backgrounds/sclera should NOT be tinted - they stay white/greyscale
        let lower_name = node_name.to_lowercase();
        if lower_name.contains("background") || lower_name.contains("sclera") {
            return None;
        }

        // Check for eye-related nodes (iris/pupil area)
        if node_name.contains("Eye")
            && !node_name.contains("Eyelid")
            && !node_name.contains("Eyebrow")
        {
            if let Some(ref eyes) = self.eyes {
                return Some(eyes);
            }
        }

        // Check for hair-related nodes (eyebrows use hair color)
        if node_name.contains("Hair") || node_name.contains("Eyebrow") {
            if let Some(ref hair) = self.hair {
                return Some(hair);
            }
        }

        // Anything ending in -Suit is underwear related
        if node_name.ends_with("-Suit") {
            if let Some(ref underwear) = self.underwear {
                return Some(underwear);
            }
        }

        // Handle cape tinting
        if node_name.contains("Cape") {
            if let Some(ref cape) = self.cape {
                return Some(cape);
            }
        }

        // Default to skin tint for everything else (body, head, hands, etc.)
        Some(&self.skin)
    }

    /// Apply optional tints from ResolvedTints to this config
    /// This programmatically loads and assigns tints for eyes, hair, underwear, etc.
    pub fn apply_resolved_tints(&mut self, resolved: &ResolvedTints) {
        // Helper function to load an optional tint from a path
        fn apply_optional_tint(path: &Option<PathBuf>, target: &mut Option<TintGradient>) {
            if let Some(ref path) = path {
                *target = TintGradient::from_file(path).ok();
            }
        }

        apply_optional_tint(&resolved.eye_color, &mut self.eyes);
        apply_optional_tint(&resolved.hair_color, &mut self.hair);
        apply_optional_tint(&resolved.underwear_color, &mut self.underwear);
        apply_optional_tint(&resolved.cape_color, &mut self.cape);
        apply_optional_tint(&resolved.gloves_color, &mut self.gloves);
        apply_optional_tint(&resolved.head_accessory_color, &mut self.head_accessories);
        apply_optional_tint(&resolved.overpants_color, &mut self.overpants);
        apply_optional_tint(&resolved.overtop_color, &mut self.overtop);
        apply_optional_tint(&resolved.pants_color, &mut self.pants);
        apply_optional_tint(&resolved.shoes_color, &mut self.shoes);
        apply_optional_tint(&resolved.undertop_color, &mut self.undertop);
    }
}

/// Lighting configuration for adding depth through diffuse shading
#[derive(Debug, Clone, Copy)]
pub struct LightConfig {
    /// Enable lighting (disable for flat textured rendering)
    pub enabled: bool,
    /// Light direction vector (should be normalized)
    pub light_direction: Vec3,
    /// Ambient light coefficient (0.0 = fully dark shadows, 1.0 = no shadows)
    pub ambient: f32,
    /// Diffuse light coefficient (0.0 = no directional lighting, 1.0 = full contrast)
    pub diffuse: f32,
}

impl Default for LightConfig {
    fn default() -> Self {
        // Minecraft-style lighting from above and slightly forward
        LightConfig {
            enabled: true,
            light_direction: Vec3::new(0.2, 1.0, 0.3).normalize(),
            ambient: 0.85,
            diffuse: 0.5,
        }
    }
}

/// Render configuration options
#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    /// Use bilinear filtering for smoother, softer appearance (matches in-game rendering)
    pub bilinear_filtering: bool,
    /// Apply post-processing blur for anti-aliasing (0.0 = none, 1.0 = full)
    pub blur_amount: f32,
    /// Lighting configuration for depth perception
    pub light_config: LightConfig,
}

impl Default for RenderConfig {
    fn default() -> Self {
        RenderConfig {
            bilinear_filtering: false, // Use nearest-neighbor for pixel-perfect rendering
            blur_amount: 0.0,          // No blur by default
            light_config: LightConfig::default(), // Use default Minecraft-style lighting
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_direction_normalized() {
        let config = LightConfig::default();
        let length = config.light_direction.length();
        assert!(
            (length - 1.0).abs() < 0.0001,
            "Light direction should be normalized"
        );
    }

    #[test]
    fn test_lighting_coefficients_valid() {
        let config = LightConfig::default();
        assert_eq!(config.ambient, 0.85, "Default ambient should be 0.85");
        assert_eq!(config.diffuse, 0.5, "Default diffuse should be 0.5");
        assert!(
            config.ambient >= 0.0 && config.ambient <= 1.0,
            "Ambient should be in [0, 1]"
        );
        assert!(
            config.diffuse >= 0.0 && config.diffuse <= 1.0,
            "Diffuse should be in [0, 1]"
        );
    }

    #[test]
    fn test_custom_light_config() {
        let custom = LightConfig {
            enabled: false,
            light_direction: Vec3::new(1.0, 0.0, 0.0).normalize(),
            ambient: 0.3,
            diffuse: 0.7,
        };

        assert!(!custom.enabled);
        assert_eq!(custom.ambient, 0.3);
        assert_eq!(custom.diffuse, 0.7);
    }
}
