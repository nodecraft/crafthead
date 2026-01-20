//! Blockymodel data structures and parsing

use crate::error::{Error, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BlockyModel {
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub lod: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub position: Vector3,
    pub orientation: Quaternion,
    #[serde(default)]
    pub shape: Option<Shape>,
    #[serde(default)]
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn zero() -> Self {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn identity() -> Self {
        Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Shape {
    #[serde(default = "Vector3::zero")]
    pub offset: Vector3,
    #[serde(default = "default_stretch")]
    pub stretch: Vector3,
    #[serde(default, rename = "textureLayout")]
    pub texture_layout: TextureLayout,
    #[serde(rename = "type")]
    pub shape_type: ShapeType,
    pub settings: ShapeSettings,
    #[serde(default = "default_unwrap_mode", rename = "unwrapMode")]
    pub unwrap_mode: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default, rename = "doubleSided")]
    pub double_sided: bool,
    #[serde(default = "default_shading_mode", rename = "shadingMode")]
    pub shading_mode: String,
}

fn default_stretch() -> Vector3 {
    Vector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    }
}

fn default_unwrap_mode() -> String {
    "custom".to_string()
}

fn default_true() -> bool {
    true
}

fn default_shading_mode() -> String {
    "flat".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeType {
    Box,
    Quad,
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShapeSettings {
    #[serde(default, deserialize_with = "deserialize_optional_size")]
    pub size: Option<Vector3>,
    #[serde(default)]
    pub normal: Option<QuadNormal>,
    #[serde(default)]
    pub is_piece: Option<bool>,
    #[serde(default, rename = "isStaticBox")]
    pub is_static_box: Option<bool>,
}

fn deserialize_optional_size<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vector3>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, MapAccess, Visitor};
    use std::fmt;

    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Option<Vector3>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a size object with x, y, and optionally z")
        }

        fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(Some(deserialize_size(deserializer)?))
        }

        fn visit_map<M>(self, mut map: M) -> std::result::Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut x = None;
            let mut y = None;
            let mut z = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "x" => {
                        if x.is_some() {
                            return Err(de::Error::duplicate_field("x"));
                        }
                        x = Some(map.next_value()?);
                    }
                    "y" => {
                        if y.is_some() {
                            return Err(de::Error::duplicate_field("y"));
                        }
                        y = Some(map.next_value()?);
                    }
                    "z" => {
                        if z.is_some() {
                            return Err(de::Error::duplicate_field("z"));
                        }
                        z = Some(map.next_value()?);
                    }
                    _ => {
                        let _ = map.next_value::<de::IgnoredAny>()?;
                    }
                }
            }

            let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
            let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
            let z = z.unwrap_or(0.0);

            Ok(Some(Vector3 { x, y, z }))
        }
    }

    deserializer.deserialize_option(SizeVisitor)
}

fn deserialize_size<'de, D>(deserializer: D) -> std::result::Result<Vector3, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, MapAccess, Visitor};
    use std::fmt;

    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Vector3;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a size object with x, y, and optionally z")
        }

        fn visit_map<M>(self, mut map: M) -> std::result::Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut x = None;
            let mut y = None;
            let mut z = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "x" => {
                        if x.is_some() {
                            return Err(de::Error::duplicate_field("x"));
                        }
                        x = Some(map.next_value()?);
                    }
                    "y" => {
                        if y.is_some() {
                            return Err(de::Error::duplicate_field("y"));
                        }
                        y = Some(map.next_value()?);
                    }
                    "z" => {
                        if z.is_some() {
                            return Err(de::Error::duplicate_field("z"));
                        }
                        z = Some(map.next_value()?);
                    }
                    _ => {
                        let _ = map.next_value::<de::IgnoredAny>()?;
                    }
                }
            }

            let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
            let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
            let z = z.unwrap_or(0.0);

            Ok(Vector3 { x, y, z })
        }
    }

    deserializer.deserialize_map(SizeVisitor)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuadNormal {
    #[serde(rename = "+X")]
    PosX,
    #[serde(rename = "-X")]
    NegX,
    #[serde(rename = "+Y")]
    PosY,
    #[serde(rename = "-Y")]
    NegY,
    #[serde(rename = "+Z")]
    PosZ,
    #[serde(rename = "-Z")]
    NegZ,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TextureLayout {
    #[serde(default)]
    pub front: Option<UvFace>,
    #[serde(default)]
    pub back: Option<UvFace>,
    #[serde(default)]
    pub left: Option<UvFace>,
    #[serde(default)]
    pub right: Option<UvFace>,
    #[serde(default)]
    pub top: Option<UvFace>,
    #[serde(default)]
    pub bottom: Option<UvFace>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct UvFace {
    pub offset: UvOffset,
    pub mirror: UvMirror,
    pub angle: UvAngle,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct UvOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct UvMirror {
    pub x: bool,
    pub y: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct UvAngle(pub u32);

impl UvAngle {
    pub fn as_degrees(&self) -> u32 {
        self.0
    }

    pub fn as_radians(&self) -> f32 {
        self.0 as f32 * std::f32::consts::PI / 180.0
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockyAnimation {
    pub duration: u32,
    #[serde(default)]
    pub hold_last_keyframe: bool,
    pub node_animations: std::collections::HashMap<String, NodeAnimation>,
    #[serde(default)]
    pub format_version: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeAnimation {
    #[serde(default)]
    pub position: Vec<PositionKeyframe>,
    #[serde(default)]
    pub orientation: Vec<OrientationKeyframe>,
    #[serde(default)]
    pub shape_stretch: Vec<StretchKeyframe>,
    #[serde(default)]
    pub shape_visible: Vec<VisibilityKeyframe>,
    #[serde(default)]
    pub shape_uv_offset: Vec<UvOffsetKeyframe>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionKeyframe {
    pub time: u32,
    pub delta: Vector3,
    #[serde(default)]
    pub interpolation_type: InterpolationType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrientationKeyframe {
    pub time: u32,
    pub delta: Quaternion,
    #[serde(default)]
    pub interpolation_type: InterpolationType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StretchKeyframe {
    pub time: u32,
    pub delta: Vector3,
    #[serde(default)]
    pub interpolation_type: InterpolationType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityKeyframe {
    pub time: u32,
    pub delta: bool,
    #[serde(default)]
    pub interpolation_type: InterpolationType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UvOffsetKeyframe {
    pub time: u32,
    pub delta: UvOffset,
    #[serde(default)]
    pub interpolation_type: InterpolationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationType {
    #[default]
    Smooth,
    Linear,
    Step,
}

pub fn parse_blockymodel(json: &str) -> Result<BlockyModel> {
    serde_json::from_str(json)
        .map_err(|e| Error::Parse(format!("Failed to parse blockymodel JSON: {}", e)))
}

pub fn parse_blockymodel_from_file(path: &std::path::Path) -> Result<BlockyModel> {
    let contents = std::fs::read_to_string(path)?;
    parse_blockymodel(&contents)
}

pub fn parse_blockyanim(json: &str) -> Result<BlockyAnimation> {
    serde_json::from_str(json)
        .map_err(|e| Error::Parse(format!("Failed to parse blockyanim JSON: {}", e)))
}

pub fn parse_blockyanim_from_file(path: &std::path::Path) -> Result<BlockyAnimation> {
    let contents = std::fs::read_to_string(path)?;
    parse_blockyanim(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_blockymodel() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Root",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        assert_eq!(model.nodes.len(), 1);
        assert_eq!(model.nodes[0].id, "0");
        assert_eq!(model.nodes[0].name, "Root");
        assert_eq!(model.nodes[0].children.len(), 0);
    }

    #[test]
    fn test_parse_with_optional_fields() {
        let json = r#"
        {
            "lod": "auto",
            "format": "character",
            "nodes": [
                {
                    "id": "1",
                    "name": "Test",
                    "position": {"x": 1, "y": 2, "z": 3},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        assert_eq!(model.lod, Some("auto".to_string()));
        assert_eq!(model.format, Some("character".to_string()));
    }

    #[test]
    fn test_parse_node_with_shape_box() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Box",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 32, "y": 32, "z": 32}
                        },
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let node = &model.nodes[0];
        assert!(node.shape.is_some());
        let shape = node.shape.as_ref().unwrap();
        assert_eq!(shape.shape_type, ShapeType::Box);
        assert!(shape.settings.size.is_some());
        let size = shape.settings.size.unwrap();
        assert_eq!(size.x, 32.0);
        assert_eq!(size.y, 32.0);
        assert_eq!(size.z, 32.0);
    }

    #[test]
    fn test_parse_node_with_shape_quad() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Quad",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "quad",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 12, "y": 7},
                            "normal": "+Z"
                        },
                        "textureLayout": {
                            "front": {
                                "offset": {"x": 1, "y": 1},
                                "mirror": {"x": false, "y": false},
                                "angle": 0
                            }
                        },
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let node = &model.nodes[0];
        let shape = node.shape.as_ref().unwrap();
        assert_eq!(shape.shape_type, ShapeType::Quad);
        assert_eq!(shape.settings.normal, Some(QuadNormal::PosZ));
        assert!(shape.texture_layout.front.is_some());
    }

    #[test]
    fn test_parse_node_with_children() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Parent",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": [
                        {
                            "id": "1",
                            "name": "Child",
                            "position": {"x": 1, "y": 1, "z": 1},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        assert_eq!(model.nodes[0].children.len(), 1);
        assert_eq!(model.nodes[0].children[0].name, "Child");
    }

    #[test]
    fn test_parse_texture_layout() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Box",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 32, "y": 32, "z": 32}
                        },
                        "textureLayout": {
                            "front": {
                                "offset": {"x": 0, "y": 0},
                                "mirror": {"x": false, "y": false},
                                "angle": 0
                            },
                            "back": {
                                "offset": {"x": 64, "y": 0},
                                "mirror": {"x": true, "y": false},
                                "angle": 90
                            }
                        },
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let shape = model.nodes[0].shape.as_ref().unwrap();
        assert!(shape.texture_layout.front.is_some());
        assert!(shape.texture_layout.back.is_some());

        let front = shape.texture_layout.front.unwrap();
        assert_eq!(front.offset.x, 0.0);
        assert_eq!(front.offset.y, 0.0);
        assert!(!front.mirror.x);
        assert_eq!(front.angle.as_degrees(), 0);

        let back = shape.texture_layout.back.unwrap();
        assert_eq!(back.offset.x, 64.0);
        assert!(back.mirror.x);
        assert_eq!(back.angle.as_degrees(), 90);
    }

    #[test]
    fn test_parse_empty_texture_layout() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Box",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 32, "y": 32, "z": 32}
                        },
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let shape = model.nodes[0].shape.as_ref().unwrap();
        assert!(shape.texture_layout.front.is_none());
    }

    #[test]
    fn test_parse_shape_none() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Group",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "none",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {},
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let shape = model.nodes[0].shape.as_ref().unwrap();
        assert_eq!(shape.shape_type, ShapeType::None);
    }

    #[test]
    fn test_parse_quaternion_identity() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Node",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": []
                }
            ]
        }
        "#;

        let model = parse_blockymodel(json).unwrap();
        let quat = model.nodes[0].orientation;
        assert_eq!(quat.x, 0.0);
        assert_eq!(quat.y, 0.0);
        assert_eq!(quat.z, 0.0);
        assert_eq!(quat.w, 1.0);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"invalid": "json"}"#;
        let result = parse_blockymodel(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let json = r#"
        {
            "nodes": [
                {
                    "id": "1"
                }
            ]
        }
        "#;
        let result = parse_blockymodel(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_player_blockymodel() {
        // Test parsing the actual Player.blockymodel file
        let path = std::path::Path::new("tests/fixtures/Player.blockymodel");
        if path.exists() {
            let model = parse_blockymodel_from_file(path).unwrap();
            assert!(!model.nodes.is_empty());
            // Player model should have nested children
            assert!(model.nodes[0].children.len() > 0);
        }
    }

    #[test]
    fn test_parse_empty_cube_blockymodel() {
        // Test parsing the simpler Empty_Cube.blockymodel file
        let path = std::path::Path::new("tests/fixtures/Empty_Cube.blockymodel");
        if path.exists() {
            let model = parse_blockymodel_from_file(path).unwrap();
            assert_eq!(model.nodes.len(), 1);
            assert!(model.nodes[0].shape.is_some());
            let shape = model.nodes[0].shape.as_ref().unwrap();
            assert_eq!(shape.shape_type, ShapeType::Box);
        }
    }

    // ==========================================================================
    // Animation Parsing Tests
    // ==========================================================================

    #[test]
    fn test_parse_minimal_blockyanim() {
        let json = r#"
        {
            "duration": 60,
            "holdLastKeyframe": false,
            "nodeAnimations": {},
            "formatVersion": 1
        }
        "#;

        let anim = parse_blockyanim(json).unwrap();
        assert_eq!(anim.duration, 60);
        assert!(!anim.hold_last_keyframe);
        assert!(anim.node_animations.is_empty());
        assert_eq!(anim.format_version, Some(1));
    }

    #[test]
    fn test_parse_blockyanim_with_position_keyframes() {
        let json = r#"
        {
            "duration": 30,
            "nodeAnimations": {
                "Pelvis": {
                    "position": [
                        { "time": 0, "delta": { "x": 0, "y": -0.925, "z": -0.15 }, "interpolationType": "smooth" },
                        { "time": 30, "delta": { "x": 0, "y": -0.5, "z": 0 }, "interpolationType": "smooth" }
                    ],
                    "orientation": [],
                    "shapeStretch": [],
                    "shapeVisible": [],
                    "shapeUvOffset": []
                }
            }
        }
        "#;

        let anim = parse_blockyanim(json).unwrap();
        assert_eq!(anim.duration, 30);

        let pelvis = anim.node_animations.get("Pelvis").unwrap();
        assert_eq!(pelvis.position.len(), 2);
        assert_eq!(pelvis.position[0].time, 0);
        assert_eq!(pelvis.position[0].delta.y, -0.925);
        assert_eq!(
            pelvis.position[0].interpolation_type,
            InterpolationType::Smooth
        );
        assert_eq!(pelvis.position[1].time, 30);
    }

    #[test]
    fn test_parse_blockyanim_with_orientation_keyframes() {
        let json = r#"
        {
            "duration": 60,
            "nodeAnimations": {
                "R-Thigh": {
                    "position": [],
                    "orientation": [
                        { "time": 0, "delta": { "x": -0.021637, "y": -0.006414, "z": -0.026289, "w": 0.9994 }, "interpolationType": "smooth" },
                        { "time": 30, "delta": { "x": 0.000169, "y": -0.00546, "z": -0.026107, "w": 0.999644 }, "interpolationType": "smooth" }
                    ],
                    "shapeStretch": [],
                    "shapeVisible": [],
                    "shapeUvOffset": []
                }
            }
        }
        "#;

        let anim = parse_blockyanim(json).unwrap();
        let thigh = anim.node_animations.get("R-Thigh").unwrap();
        assert_eq!(thigh.orientation.len(), 2);
        assert_eq!(thigh.orientation[0].time, 0);
        assert!((thigh.orientation[0].delta.x - (-0.021637)).abs() < 0.0001);
        assert!((thigh.orientation[0].delta.w - 0.9994).abs() < 0.0001);
    }

    #[test]
    fn test_parse_blockyanim_interpolation_types() {
        let json = r#"
        {
            "duration": 60,
            "nodeAnimations": {
                "Test": {
                    "position": [
                        { "time": 0, "delta": { "x": 0, "y": 0, "z": 0 }, "interpolationType": "smooth" },
                        { "time": 20, "delta": { "x": 1, "y": 1, "z": 1 }, "interpolationType": "linear" },
                        { "time": 40, "delta": { "x": 2, "y": 2, "z": 2 }, "interpolationType": "step" }
                    ],
                    "orientation": [],
                    "shapeStretch": [],
                    "shapeVisible": [],
                    "shapeUvOffset": []
                }
            }
        }
        "#;

        let anim = parse_blockyanim(json).unwrap();
        let test = anim.node_animations.get("Test").unwrap();
        assert_eq!(
            test.position[0].interpolation_type,
            InterpolationType::Smooth
        );
        assert_eq!(
            test.position[1].interpolation_type,
            InterpolationType::Linear
        );
        assert_eq!(test.position[2].interpolation_type, InterpolationType::Step);
    }

    #[test]
    fn test_parse_idle_blockyanim_file() {
        // Test parsing the actual Idle.blockyanim file
        let path =
            std::path::Path::new("assets/Common/Characters/Animations/Default/Idle.blockyanim");
        if path.exists() {
            let anim = parse_blockyanim_from_file(path).unwrap();
            assert_eq!(anim.duration, 60);

            // Should have animations for various body parts
            assert!(anim.node_animations.contains_key("Pelvis"));
            assert!(anim.node_animations.contains_key("R-Thigh"));
            assert!(anim.node_animations.contains_key("L-Thigh"));
            assert!(anim.node_animations.contains_key("Chest"));
            assert!(anim.node_animations.contains_key("Head"));

            // Pelvis should have position keyframes
            let pelvis = anim.node_animations.get("Pelvis").unwrap();
            assert!(!pelvis.position.is_empty());
        }
    }
}
