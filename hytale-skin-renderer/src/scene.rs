//! Scene graph construction and transformation handling

use crate::animation::AnimationPose;
use crate::error::Result;
use crate::math::{build_transform_with_offset, multiply_transforms};
use crate::models::{BlockyModel, Node, Shape, ShapeType};
use glam::Mat4;

/// A scene node with its transformation matrix
#[derive(Debug, Clone)]
pub struct SceneNode {
	pub id: String,
	pub name: String,
	pub transform: Mat4,
	pub shape: Option<Shape>,
	pub children: Vec<SceneNode>,
	pub texture_id: Option<usize>,
}

/// Scene graph built from a blockymodel
#[derive(Debug)]
pub struct SceneGraph {
	pub nodes: Vec<SceneNode>,
}

/// Configuration for joint spacing in static renders
#[derive(Debug, Clone)]
pub struct JointSpacingConfig {
	/// Enable joint spacing adjustments
	pub enabled: bool,

	/// Use automatic overlap detection instead of hardcoded values
	pub auto_detect: bool,

	/// Extra spacing beyond detected overlap (can be negative for less gap)
	/// Only used when auto_detect is true
	pub extra_spacing: f32,

	/// Manual overrides for specific joints (takes precedence over auto-detection)
	/// Key format: "ParentName->ChildName" (e.g., "R-Thigh->R-Calf")
	pub manual_overrides: std::collections::HashMap<String, f32>,

	// Legacy fields (kept for backward compatibility)
	pub pelvis_to_thigh: f32,
	pub thigh_to_calf: f32,
	pub calf_to_foot: f32,
}

impl Default for JointSpacingConfig {
	fn default() -> Self {
		Self {
			enabled: false,
			auto_detect: true,  // Use auto-detection by default when enabled
			extra_spacing: 0.0, // No extra gap, just eliminate overlap
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 0.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		}
	}
}

impl JointSpacingConfig {
	/// Create config with automatic overlap detection
	pub fn auto() -> Self {
		Self {
			enabled: true,
			auto_detect: true,
			extra_spacing: 0.0,
			..Default::default()
		}
	}

	/// Create config with auto-detection and extra gap
	pub fn auto_with_gap(extra_spacing: f32) -> Self {
		Self {
			enabled: true,
			auto_detect: true,
			extra_spacing,
			..Default::default()
		}
	}

	/// Add a manual override for a specific joint
	pub fn with_override(mut self, parent_name: &str, child_name: &str, spacing: f32) -> Self {
		let key = format!("{}->{}", parent_name, child_name);
		self.manual_overrides.insert(key, spacing);
		self
	}
}

impl SceneGraph {
	pub fn from_blockymodel(model: &BlockyModel) -> Result<Self> {
		Self::from_blockymodel_with_config(model, None)
	}

	pub fn from_blockymodel_with_config(
		model: &BlockyModel,
		config: Option<&JointSpacingConfig>,
	) -> Result<Self> {
		let mut nodes = Vec::new();

		for node in &model.nodes {
			let scene_node = build_scene_node(node, Mat4::IDENTITY, None, config, None)?;
			nodes.push(scene_node);
		}

		Ok(SceneGraph { nodes })
	}

	/// Build a scene graph with animation pose applied
	pub fn from_blockymodel_with_pose(
		model: &BlockyModel,
		pose: &AnimationPose,
		config: Option<&JointSpacingConfig>,
	) -> Result<Self> {
		let mut nodes = Vec::new();

		for node in &model.nodes {
			let scene_node = build_scene_node(node, Mat4::IDENTITY, None, config, Some(pose))?;
			nodes.push(scene_node);
		}

		Ok(SceneGraph { nodes })
	}

	pub fn get_visible_shapes(&self) -> Vec<(&SceneNode, Mat4)> {
		let mut result = Vec::new();
		for node in &self.nodes {
			collect_visible_shapes(node, Mat4::IDENTITY, &mut result);
		}
		result
	}

	/// Merge another scene graph into this one, tagging new nodes with the given texture ID
	pub fn merge_graph(&mut self, other: SceneGraph, texture_id: usize) {
		for node in other.nodes {
			Self::merge_node_recursive(&mut self.nodes, node, texture_id);
		}
	}

	fn merge_node_recursive(nodes: &mut Vec<SceneNode>, mut node: SceneNode, texture_id: usize) {
		// Try to find a matching node by name
		if let Some(existing) = nodes.iter_mut().find(|n| n.name == node.name) {
			// Match found! Merge children into existing node
			for child in node.children {
				Self::merge_node_recursive(&mut existing.children, child, texture_id);
			}
		} else {
			// No match found, add this node as a new sibling
			// Apply the texture ID to this node and all its children
			Self::apply_texture_id_recursive(&mut node, texture_id);
			nodes.push(node);
		}
	}

	fn apply_texture_id_recursive(node: &mut SceneNode, texture_id: usize) {
		node.texture_id = Some(texture_id);
		for child in &mut node.children {
			Self::apply_texture_id_recursive(child, texture_id);
		}
	}
}

/// Calculate Y-axis bounds of a shape in its local space
fn calculate_shape_y_bounds(shape: &Shape) -> Option<(f32, f32)> {
	if shape.shape_type != ShapeType::Box {
		return None;
	}

	let size = shape.settings.size?;
	let offset = shape.offset;
	let stretch = shape.stretch;

	let half_height = (size.y / 2.0) * stretch.y.abs();

	let min_y = offset.y - half_height;
	let max_y = offset.y + half_height;

	Some((min_y, max_y))
}

/// Calculate Y-axis overlap between parent shape and child node
fn calculate_y_overlap(parent_shape: &Shape, child_node: &Node) -> f32 {
	let parent_bounds = match calculate_shape_y_bounds(parent_shape) {
		Some(bounds) => bounds,
		None => return 0.0,
	};

	let child_shape = match &child_node.shape {
		Some(shape) => shape,
		None => return 0.0,
	};

	let child_bounds = match calculate_shape_y_bounds(child_shape) {
		Some(bounds) => bounds,
		None => return 0.0,
	};

	// Calculate in parent's coordinate space
	let parent_bottom = parent_bounds.0;
	let child_top_in_parent_space = child_node.position.y + child_bounds.1;

	let overlap = child_top_in_parent_space - parent_bottom;
	overlap.max(0.0)
}

/// Calculate joint spacing based on parent and child nodes
fn calculate_joint_spacing(
	parent_node: Option<&Node>,
	child_node: &Node,
	config: Option<&JointSpacingConfig>,
) -> f32 {
	let config = match config {
		Some(c) if c.enabled => c,
		_ => return 0.0,
	};

	let parent = match parent_node {
		Some(p) => p,
		None => return 0.0, // No parent, no spacing
	};

	// Check for manual override first (highest priority)
	let override_key = format!("{}->{}", parent.name, child_node.name);
	if let Some(&spacing) = config.manual_overrides.get(&override_key) {
		return spacing;
	}

	// Auto-detection mode
	if config.auto_detect {
		let parent_shape = match &parent.shape {
			Some(shape) => shape,
			None => return 0.0, // Parent has no shape
		};

		let overlap = calculate_y_overlap(parent_shape, child_node);
		return overlap + config.extra_spacing;
	}

	// Legacy mode: pattern-based with hardcoded values
	calculate_joint_spacing_legacy(&parent.name, &child_node.name, config)
}

/// Multiply two quaternions (Hamilton product)
/// Result = q1 * q2 (apply q2's rotation then q1's rotation)
fn multiply_quaternions(
	q1: &crate::models::Quaternion,
	q2: &crate::models::Quaternion,
) -> crate::models::Quaternion {
	crate::models::Quaternion {
		w: q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z,
		x: q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
		y: q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
		z: q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
	}
}

/// Legacy pattern-based spacing (for backward compatibility)
fn calculate_joint_spacing_legacy(
	parent_name: &str,
	child_name: &str,
	config: &JointSpacingConfig,
) -> f32 {
	// Extract side prefix (R- or L-) from names
	let parent_side = if parent_name.starts_with("R-") {
		"R"
	} else if parent_name.starts_with("L-") {
		"L"
	} else {
		""
	};

	let child_side = if child_name.starts_with("R-") {
		"R"
	} else if child_name.starts_with("L-") {
		"L"
	} else {
		""
	};

	// Only apply spacing if sides match
	if !parent_side.is_empty() && !child_side.is_empty() && parent_side != child_side {
		return 0.0;
	}

	// Match joint patterns
	if parent_name.contains("Pelvis") && child_name.contains("Thigh") {
		config.pelvis_to_thigh
	} else if parent_name.contains("Thigh") && child_name.contains("Calf") {
		config.thigh_to_calf
	} else if parent_name.contains("Calf") && child_name.contains("Foot") {
		config.calf_to_foot
	} else {
		0.0
	}
}

fn build_scene_node(
	node: &Node,
	parent_transform: Mat4,
	parent_node: Option<&Node>,
	config: Option<&JointSpacingConfig>,
	pose: Option<&AnimationPose>,
) -> Result<SceneNode> {
	let (position_delta, orientation_delta) = if let Some(p) = pose {
		if let Some(node_pose) = p.get(&node.name) {
			(node_pose.position_delta, node_pose.orientation_delta)
		} else {
			(None, None)
		}
	} else {
		(None, None)
	};

	let final_position = if let Some(delta) = position_delta {
		Vector3 {
			x: node.position.x + delta.x,
			y: node.position.y + delta.y,
			z: node.position.z + delta.z,
		}
	} else {
		node.position
	};

	let final_orientation = if let Some(delta) = orientation_delta {
		multiply_quaternions(&node.orientation, &delta)
	} else {
		node.orientation
	};

	let node_transform = build_transform_with_offset(
		final_position,
		final_orientation,
		Vector3 {
			x: 1.0,
			y: 1.0,
			z: 1.0,
		},
		Vector3::zero(),
	);

	let mut world_transform = multiply_transforms(parent_transform, node_transform);

	// Apply joint spacing only when NO animation pose is provided
	if pose.is_none() {
		let spacing = calculate_joint_spacing(parent_node, node, config);
		if spacing > 0.0 {
			use glam::Vec3;
			world_transform =
				world_transform * Mat4::from_translation(Vec3::new(0.0, -spacing, 0.0));
		}
	}

	let shape = if let Some(ref s) = node.shape {
		if s.visible {
			Some(s.clone())
		} else {
			None
		}
	} else {
		None
	};

	// NOTE: Shape stretch is applied in geometry generation.
	// However, shape OFFSET affects where children are positioned - children are positioned
	// relative to the parent's SHAPE CENTER (pivot + offset), not the pivot itself.
	// This matches the Blockbench plugin behavior.

	let parent_offset = if let Some(ref s) = node.shape {
		glam::Vec3::new(s.offset.x, s.offset.y, s.offset.z)
	} else {
		glam::Vec3::ZERO
	};
	let child_parent_transform = world_transform * Mat4::from_translation(parent_offset);

	let mut children = Vec::new();
	for child in &node.children {
		let child_node = build_scene_node(child, child_parent_transform, Some(node), config, pose)?;
		children.push(child_node);
	}

	Ok(SceneNode {
		id: node.id.clone(),
		name: node.name.clone(),
		transform: world_transform,
		shape,
		children,
		texture_id: None,
	})
}

fn collect_visible_shapes<'a>(
	node: &'a SceneNode,
	_transform: Mat4,
	result: &mut Vec<(&'a SceneNode, Mat4)>,
) {
	if let Some(ref shape) = node.shape {
		if shape.visible {
			result.push((node, node.transform));
		}
	}

	for child in &node.children {
		collect_visible_shapes(child, node.transform, result);
	}
}

// Helper to convert Vector3 - we'll use the math module
use crate::models::Vector3;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::{parse_blockymodel, ShapeSettings};

	#[test]
	fn test_build_scene_graph_single_node() {
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
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		assert_eq!(scene.nodes.len(), 1);
		assert_eq!(scene.nodes[0].name, "Root");
	}

	#[test]
	fn test_build_scene_graph_with_nested_children() {
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
                            "position": {"x": 1, "y": 2, "z": 3},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": [
                                {
                                    "id": "2",
                                    "name": "Grandchild",
                                    "position": {"x": 10, "y": 20, "z": 30},
                                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		assert_eq!(scene.nodes.len(), 1);
		assert_eq!(scene.nodes[0].children.len(), 1);
		assert_eq!(scene.nodes[0].children[0].name, "Child");
		assert_eq!(scene.nodes[0].children[0].children.len(), 1);
		assert_eq!(scene.nodes[0].children[0].children[0].name, "Grandchild");
	}

	#[test]
	fn test_apply_parent_transforms_to_children() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Parent",
                    "position": {"x": 10, "y": 20, "z": 30},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": [
                        {
                            "id": "1",
                            "name": "Child",
                            "position": {"x": 1, "y": 2, "z": 3},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		// Child's world position should be parent + child local position
		let child = &scene.nodes[0].children[0];
		let origin = Vector3::zero();
		let world_pos = crate::math::transform_point(child.transform, origin);

		// Should be at parent position + child position
		assert!((world_pos.x - 11.0).abs() < 0.1);
		assert!((world_pos.y - 22.0).abs() < 0.1);
		assert!((world_pos.z - 33.0).abs() < 0.1);
	}

	#[test]
	fn test_handle_invisible_nodes() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Visible",
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
                },
                {
                    "id": "1",
                    "name": "Invisible",
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
                        "visible": false,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": []
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let visible_shapes = scene.get_visible_shapes();
		assert_eq!(visible_shapes.len(), 1);
		assert_eq!(visible_shapes[0].0.name, "Visible");
	}

	#[test]
	fn test_transform_chain() {
		// Test grandparent -> parent -> child transform chain
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Grandparent",
                    "position": {"x": 100, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": [
                        {
                            "id": "1",
                            "name": "Parent",
                            "position": {"x": 10, "y": 0, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": [
                                {
                                    "id": "2",
                                    "name": "Child",
                                    "position": {"x": 1, "y": 0, "z": 0},
                                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let child = &scene.nodes[0].children[0].children[0];
		let origin = Vector3::zero();
		let world_pos = crate::math::transform_point(child.transform, origin);

		// Should be at 100 + 10 + 1 = 111
		assert!((world_pos.x - 111.0).abs() < 0.1);
	}

	#[test]
	fn test_stretch_transformations() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Stretched",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 2, "y": 3, "z": 4},
                        "settings": {
                            "size": {"x": 10, "y": 10, "z": 10}
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
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		// Stretch should be stored in shape, transform should account for it
		let node = &scene.nodes[0];
		assert!(node.shape.is_some());
		let shape = node.shape.as_ref().unwrap();
		assert!((shape.stretch.x - 2.0).abs() < 0.001);
		assert!((shape.stretch.y - 3.0).abs() < 0.001);
		assert!((shape.stretch.z - 4.0).abs() < 0.001);
	}

	#[test]
	fn test_parent_shape_offset_affects_child() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "1",
                    "name": "Parent",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 10, "y": 20, "z": 30},
                        "settings": {"size": {"x": 1, "y": 1, "z": 1}},
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": [
                        {
                            "id": "2",
                            "name": "Child",
                            "position": {"x": 5, "y": 0, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let parent = &scene.nodes[0];
		let child = &parent.children[0];

		let origin = Vector3::zero();
		let child_pos = crate::math::transform_point(child.transform, origin);

		// Child is at (5, 0, 0) relative to Parent SHAPE center (which is 10, 20, 30)
		// So child world pos should be (10+5, 20+0, 30+0) = (15, 20, 30)
		assert!(
			(child_pos.x - 15.0).abs() < 0.1,
			"Expected x=15.0, got {}",
			child_pos.x
		);
		assert!(
			(child_pos.y - 20.0).abs() < 0.1,
			"Expected y=20.0, got {}",
			child_pos.y
		);
		assert!(
			(child_pos.z - 30.0).abs() < 0.1,
			"Expected z=30.0, got {}",
			child_pos.z
		);
	}

	#[test]
	fn test_offset_transformations() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Offset",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 5, "y": 10, "z": 15},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 10, "y": 10, "z": 10}
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
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let node = &scene.nodes[0];
		let origin = Vector3::zero();
		let world_pos = crate::math::transform_point(node.transform, origin);

		// Transform should NOT include offset (offset is applied in geometry generation)
		// Node is at (0,0,0), so transform should be at (0,0,0)
		assert!((world_pos.x - 0.0).abs() < 0.1);
		assert!((world_pos.y - 0.0).abs() < 0.1);
		assert!((world_pos.z - 0.0).abs() < 0.1);

		// Verify offset is stored in shape (will be applied during geometry generation)
		if let Some(ref shape) = node.shape {
			assert!((shape.offset.x - 5.0).abs() < 0.1);
			assert!((shape.offset.y - 10.0).abs() < 0.1);
			assert!((shape.offset.z - 15.0).abs() < 0.1);
		}
	}

	#[test]
	fn test_children_positioned_relative_to_parent_shape_center() {
		// Test that children are positioned relative to parent's shape CENTER (pivot + offset)
		// This matches Blockbench plugin behavior (blockymodel.ts lines 517-524)
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Parent",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 10, "y": 20, "z": 30},
                        "stretch": {"x": 2, "y": 2, "z": 2},
                        "settings": {
                            "size": {"x": 10, "y": 10, "z": 10}
                        },
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": [
                        {
                            "id": "1",
                            "name": "Child",
                            "position": {"x": 5, "y": 5, "z": 5},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let child = &scene.nodes[0].children[0];
		let origin = Vector3::zero();
		let world_pos = crate::math::transform_point(child.transform, origin);

		// Child should be at parent_shape_center + child_position
		// Parent shape center = parent_position + offset = (0,0,0) + (10,20,30) = (10,20,30)
		// Child world position = (10,20,30) + (5,5,5) = (15,25,35)
		assert!(
			(world_pos.x - 15.0).abs() < 0.1,
			"Child X should be 15 (parent offset 10 + child pos 5)"
		);
		assert!(
			(world_pos.y - 25.0).abs() < 0.1,
			"Child Y should be 25 (parent offset 20 + child pos 5)"
		);
		assert!(
			(world_pos.z - 35.0).abs() < 0.1,
			"Child Z should be 35 (parent offset 30 + child pos 5)"
		);

		// Parent's stored transform should NOT include shape offset (offset is applied in geometry generation)
		// But children use parent's shape center for positioning
		let parent = &scene.nodes[0];
		let parent_origin = Vector3::zero();
		let parent_world_pos = crate::math::transform_point(parent.transform, parent_origin);

		// Parent transform should be at node position (0,0,0), not offset
		assert!(
			(parent_world_pos.x - 0.0).abs() < 0.1,
			"Parent transform should be at node position"
		);
		assert!(
			(parent_world_pos.y - 0.0).abs() < 0.1,
			"Parent transform should be at node position"
		);
		assert!(
			(parent_world_pos.z - 0.0).abs() < 0.1,
			"Parent transform should be at node position"
		);

		// Verify offset is stored in shape (will be applied during geometry generation)
		if let Some(ref shape) = parent.shape {
			assert!(
				(shape.offset.x - 10.0).abs() < 0.1,
				"Shape offset should be stored in shape"
			);
			assert!(
				(shape.offset.y - 20.0).abs() < 0.1,
				"Shape offset should be stored in shape"
			);
			assert!(
				(shape.offset.z - 30.0).abs() < 0.1,
				"Shape offset should be stored in shape"
			);
		}
	}

	#[test]
	fn test_get_visible_shapes_no_double_transform() {
		// Test that get_visible_shapes doesn't apply transforms twice
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Parent",
                    "position": {"x": 10, "y": 20, "z": 30},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": 0, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {
                            "size": {"x": 10, "y": 10, "z": 10}
                        },
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": [
                        {
                            "id": "1",
                            "name": "Child",
                            "position": {"x": 5, "y": 5, "z": 5},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "shape": {
                                "type": "box",
                                "offset": {"x": 0, "y": 0, "z": 0},
                                "stretch": {"x": 1, "y": 1, "z": 1},
                                "settings": {
                                    "size": {"x": 5, "y": 5, "z": 5}
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
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let visible_shapes = scene.get_visible_shapes();
		assert_eq!(visible_shapes.len(), 2);

		// Check parent transform
		let (parent_node, parent_transform) = visible_shapes
			.iter()
			.find(|(n, _)| n.name == "Parent")
			.unwrap();

		// Transform should match the node's stored transform (not double-applied)
		assert_eq!(*parent_transform, parent_node.transform);

		// Check child transform
		let (child_node, child_transform) = visible_shapes
			.iter()
			.find(|(n, _)| n.name == "Child")
			.unwrap();

		// Transform should match the node's stored transform (not double-applied)
		assert_eq!(*child_transform, child_node.transform);

		// Verify child position is correct (parent position + child position)
		let origin = Vector3::zero();
		let child_world_pos = crate::math::transform_point(*child_transform, origin);

		// Should be at (10+5, 20+5, 30+5) = (15, 25, 35)
		assert!(
			(child_world_pos.x - 15.0).abs() < 0.1,
			"Child X should be 15 (10+5), not double-transformed"
		);
		assert!(
			(child_world_pos.y - 25.0).abs() < 0.1,
			"Child Y should be 25 (20+5), not double-transformed"
		);
		assert!(
			(child_world_pos.z - 35.0).abs() < 0.1,
			"Child Z should be 35 (30+5), not double-transformed"
		);
	}

	#[test]
	fn test_get_visible_shapes() {
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Box1",
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
                    "children": [
                        {
                            "id": "1",
                            "name": "Box2",
                            "position": {"x": 0, "y": 0, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "shape": {
                                "type": "box",
                                "offset": {"x": 0, "y": 0, "z": 0},
                                "stretch": {"x": 1, "y": 1, "z": 1},
                                "settings": {
                                    "size": {"x": 16, "y": 16, "z": 16}
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
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		let visible_shapes = scene.get_visible_shapes();
		assert_eq!(visible_shapes.len(), 2);
		assert_eq!(visible_shapes[0].0.name, "Box1");
		assert_eq!(visible_shapes[1].0.name, "Box2");
	}

	#[test]
	fn test_calculate_joint_spacing_thigh_to_calf() {
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		let spacing = calculate_joint_spacing_legacy("R-Thigh", "R-Calf", &config);
		assert_eq!(spacing, 12.0);
	}

	#[test]
	fn test_calculate_joint_spacing_calf_to_foot() {
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		let spacing = calculate_joint_spacing_legacy("L-Calf", "L-Foot", &config);
		assert_eq!(spacing, 10.0);
	}

	#[test]
	fn test_calculate_joint_spacing_pelvis_to_thigh() {
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		let spacing = calculate_joint_spacing_legacy("Pelvis", "R-Thigh", &config);
		assert_eq!(spacing, 5.0);
	}

	#[test]
	fn test_calculate_joint_spacing_non_joint_pair() {
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		let spacing = calculate_joint_spacing_legacy("Head", "Torso", &config);
		assert_eq!(spacing, 0.0);
	}

	#[test]
	fn test_calculate_joint_spacing_mismatched_sides() {
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		// R-Thigh -> L-Calf should not get spacing
		let spacing = calculate_joint_spacing_legacy("R-Thigh", "L-Calf", &config);
		assert_eq!(spacing, 0.0);
	}

	#[test]
	fn test_calculate_joint_spacing_disabled() {
		let config = JointSpacingConfig {
			enabled: false,
			auto_detect: false,
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 5.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		// Test that when enabled=false, the main function returns 0.0
		let parent_node = Node {
			id: "parent".to_string(),
			name: "R-Thigh".to_string(),
			position: Vector3::zero(),
			orientation: crate::models::Quaternion::identity(),
			shape: None,
			children: vec![],
		};

		let child_node = Node {
			id: "child".to_string(),
			name: "R-Calf".to_string(),
			position: Vector3::zero(),
			orientation: crate::models::Quaternion::identity(),
			shape: None,
			children: vec![],
		};

		let spacing = calculate_joint_spacing(Some(&parent_node), &child_node, Some(&config));
		assert_eq!(spacing, 0.0);
	}

	#[test]
	fn test_joint_spacing_transform_propagation() {
		// Test that spacing on Calf automatically moves Foot through transform chain
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "R-Thigh",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": [
                        {
                            "id": "1",
                            "name": "R-Calf",
                            "position": {"x": 0, "y": 0, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": [
                                {
                                    "id": "2",
                                    "name": "R-Foot",
                                    "position": {"x": 0, "y": 0, "z": 0},
                                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Use legacy mode
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 0.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel_with_config(&model, Some(&config)).unwrap();

		let thigh = &scene.nodes[0];
		let calf = &thigh.children[0];
		let foot = &calf.children[0];

		// Thigh should be at origin
		let thigh_pos = crate::math::transform_point(thigh.transform, Vector3::zero());
		assert!((thigh_pos.y - 0.0).abs() < 0.1);

		// Calf should be moved down by 12 (thigh_to_calf spacing)
		let calf_pos = crate::math::transform_point(calf.transform, Vector3::zero());
		assert!((calf_pos.y - (-12.0)).abs() < 0.1);

		// Foot should be moved down by 12 + 10 = 22 total (propagation)
		let foot_pos = crate::math::transform_point(foot.transform, Vector3::zero());
		assert!((foot_pos.y - (-22.0)).abs() < 0.1);
	}

	#[test]
	fn test_from_blockymodel_with_config_backward_compatible() {
		// Test that from_blockymodel (without config) still works
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
		let scene = SceneGraph::from_blockymodel(&model).unwrap();

		assert_eq!(scene.nodes.len(), 1);
		assert_eq!(scene.nodes[0].name, "Root");
	}

	#[test]
	fn test_calculate_shape_y_bounds_basic() {
		// Box: size.y=20, offset.y=-10, stretch.y=1
		// Expected: min_y = -10 - 10 = -20, max_y = -10 + 10 = 0
		let shape = Shape {
			shape_type: ShapeType::Box,
			offset: Vector3 {
				x: 0.0,
				y: -10.0,
				z: 0.0,
			},
			stretch: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: Some(Vector3 {
					x: 10.0,
					y: 20.0,
					z: 10.0,
				}),
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		let bounds = calculate_shape_y_bounds(&shape).unwrap();
		assert!((bounds.0 - (-20.0)).abs() < 0.1);
		assert!((bounds.1 - 0.0).abs() < 0.1);
	}

	#[test]
	fn test_calculate_shape_y_bounds_with_stretch() {
		// Box: size.y=20, offset.y=0, stretch.y=2.0
		// Expected: min_y = 0 - 20 = -20, max_y = 0 + 20 = 20
		let shape = Shape {
			shape_type: ShapeType::Box,
			offset: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			stretch: Vector3 {
				x: 1.0,
				y: 2.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: Some(Vector3 {
					x: 10.0,
					y: 20.0,
					z: 10.0,
				}),
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		let bounds = calculate_shape_y_bounds(&shape).unwrap();
		assert!((bounds.0 - (-20.0)).abs() < 0.1);
		assert!((bounds.1 - 20.0).abs() < 0.1);
	}

	#[test]
	fn test_calculate_shape_y_bounds_negative_stretch() {
		// Negative stretch (mirroring) should work same as positive
		let shape = Shape {
			shape_type: ShapeType::Box,
			offset: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			stretch: Vector3 {
				x: 1.0,
				y: -2.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: Some(Vector3 {
					x: 10.0,
					y: 20.0,
					z: 10.0,
				}),
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		let bounds = calculate_shape_y_bounds(&shape).unwrap();
		assert!((bounds.0 - (-20.0)).abs() < 0.1);
		assert!((bounds.1 - 20.0).abs() < 0.1);
	}

	#[test]
	fn test_calculate_shape_y_bounds_quad_returns_none() {
		let shape = Shape {
			shape_type: ShapeType::Quad,
			offset: Vector3::zero(),
			stretch: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: None,
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		assert!(calculate_shape_y_bounds(&shape).is_none());
	}

	#[test]
	fn test_calculate_y_overlap_real_data() {
		// Use approximate R-Thigh -> R-Calf data from Player.blockymodel
		// R-Thigh bottom: ~-21, R-Calf top in parent space: ~-10
		// Expected overlap: ~11 units

		let parent_shape = Shape {
			shape_type: ShapeType::Box,
			offset: Vector3 {
				x: 0.0,
				y: -11.0,
				z: 0.0,
			},
			stretch: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: Some(Vector3 {
					x: 10.0,
					y: 20.0,
					z: 12.0,
				}),
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		let child_node = Node {
			id: "test-calf".to_string(),
			name: "R-Calf".to_string(),
			position: Vector3 {
				x: 0.0,
				y: -10.0,
				z: 2.0,
			},
			orientation: crate::models::Quaternion {
				x: 0.0,
				y: 0.0,
				z: 0.0,
				w: 1.0,
			},
			shape: Some(Shape {
				shape_type: ShapeType::Box,
				offset: Vector3 {
					x: 0.0,
					y: -12.0,
					z: 0.0,
				},
				stretch: Vector3 {
					x: 1.0,
					y: 1.0,
					z: 1.0,
				},
				settings: ShapeSettings {
					size: Some(Vector3 {
						x: 10.0,
						y: 24.0,
						z: 12.0,
					}),
					normal: None,
					is_piece: None,
					is_static_box: None,
				},
				visible: true,
				double_sided: false,
				shading_mode: String::from("flat"),
				texture_layout: crate::models::TextureLayout::default(),
				unwrap_mode: String::from("stretch"),
			}),
			children: vec![],
		};

		let overlap = calculate_y_overlap(&parent_shape, &child_node);
		// Should be approximately 11 units
		assert!((overlap - 11.0).abs() < 0.5);
	}

	#[test]
	fn test_calculate_y_overlap_no_overlap() {
		// Parts already separated
		let parent_shape = Shape {
			shape_type: ShapeType::Box,
			offset: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			stretch: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			settings: ShapeSettings {
				size: Some(Vector3 {
					x: 10.0,
					y: 10.0,
					z: 10.0,
				}),
				normal: None,
				is_piece: None,
				is_static_box: None,
			},
			visible: true,
			double_sided: false,
			shading_mode: String::from("flat"),
			texture_layout: crate::models::TextureLayout::default(),
			unwrap_mode: String::from("stretch"),
		};

		let child_node = Node {
			id: "test-child".to_string(),
			name: "Child".to_string(),
			position: Vector3 {
				x: 0.0,
				y: -20.0,
				z: 0.0,
			}, // Far below parent
			orientation: crate::models::Quaternion {
				x: 0.0,
				y: 0.0,
				z: 0.0,
				w: 1.0,
			},
			shape: Some(Shape {
				shape_type: ShapeType::Box,
				offset: Vector3 {
					x: 0.0,
					y: 0.0,
					z: 0.0,
				},
				stretch: Vector3 {
					x: 1.0,
					y: 1.0,
					z: 1.0,
				},
				settings: ShapeSettings {
					size: Some(Vector3 {
						x: 10.0,
						y: 10.0,
						z: 10.0,
					}),
					normal: None,
					is_piece: None,
					is_static_box: None,
				},
				visible: true,
				double_sided: false,
				shading_mode: String::from("flat"),
				texture_layout: crate::models::TextureLayout::default(),
				unwrap_mode: String::from("stretch"),
			}),
			children: vec![],
		};

		let overlap = calculate_y_overlap(&parent_shape, &child_node);
		assert_eq!(overlap, 0.0);
	}

	#[test]
	fn test_config_auto_detection() {
		let config = JointSpacingConfig::auto();
		assert!(config.enabled);
		assert!(config.auto_detect);
		assert_eq!(config.extra_spacing, 0.0);
	}

	#[test]
	fn test_config_with_override() {
		let config = JointSpacingConfig::auto().with_override("R-Thigh", "R-Calf", 15.0);

		assert_eq!(config.manual_overrides.get("R-Thigh->R-Calf"), Some(&15.0));
	}

	#[test]
	fn test_auto_spacing_with_scene_graph() {
		// Build a simple parent-child hierarchy with overlapping shapes
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "Parent",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "shape": {
                        "type": "box",
                        "offset": {"x": 0, "y": -10, "z": 0},
                        "stretch": {"x": 1, "y": 1, "z": 1},
                        "settings": {"size": {"x": 10, "y": 20, "z": 10}},
                        "textureLayout": {},
                        "visible": true,
                        "doubleSided": false,
                        "shadingMode": "flat"
                    },
                    "children": [
                        {
                            "id": "1",
                            "name": "Child",
                            "position": {"x": 0, "y": -10, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "shape": {
                                "type": "box",
                                "offset": {"x": 0, "y": -10, "z": 0},
                                "stretch": {"x": 1, "y": 1, "z": 1},
                                "settings": {"size": {"x": 10, "y": 20, "z": 10}},
                                "textureLayout": {},
                                "visible": true,
                                "doubleSided": false,
                                "shadingMode": "flat"
                            },
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

		let config = JointSpacingConfig::auto();
		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel_with_config(&model, Some(&config)).unwrap();

		// Verify spacing was applied automatically
		let parent = &scene.nodes[0];
		let child = &parent.children[0];

		// Child should be offset by the detected overlap
		let child_pos = crate::math::transform_point(child.transform, Vector3::zero());
		// Expected: child moved down by overlap amount (should be > 0)
		assert!(child_pos.y < -10.0); // Child should be pushed down
	}

	#[test]
	fn test_legacy_mode_still_works() {
		// Verify backward compatibility with old config
		let config = JointSpacingConfig {
			enabled: true,
			auto_detect: false, // Disable auto-detection
			extra_spacing: 0.0,
			manual_overrides: std::collections::HashMap::new(),
			pelvis_to_thigh: 0.0,
			thigh_to_calf: 12.0,
			calf_to_foot: 10.0,
		};

		// This should use the legacy pattern matching
		let json = r#"
        {
            "nodes": [
                {
                    "id": "0",
                    "name": "R-Thigh",
                    "position": {"x": 0, "y": 0, "z": 0},
                    "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                    "children": [
                        {
                            "id": "1",
                            "name": "R-Calf",
                            "position": {"x": 0, "y": 0, "z": 0},
                            "orientation": {"x": 0, "y": 0, "z": 0, "w": 1},
                            "children": []
                        }
                    ]
                }
            ]
        }
        "#;

		let model = parse_blockymodel(json).unwrap();
		let scene = SceneGraph::from_blockymodel_with_config(&model, Some(&config)).unwrap();

		let thigh = &scene.nodes[0];
		let calf = &thigh.children[0];

		// Calf should be moved down by 12 (legacy thigh_to_calf spacing)
		let calf_pos = crate::math::transform_point(calf.transform, Vector3::zero());
		assert!((calf_pos.y - (-12.0)).abs() < 0.1);
	}
}
