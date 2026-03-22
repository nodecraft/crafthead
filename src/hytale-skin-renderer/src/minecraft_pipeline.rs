//! Minecraft skin → 3D renderable pipeline
//!
//! Converts a Minecraft skin texture into `Vec<RenderableFace>` that can be
//! rendered by the shared 3D renderer (`renderer::render_scene_tinted`).
//!
//! This allows Minecraft skins to use the same camera presets, poses, and
//! rendering pipeline as Hytale skins.

use crate::geometry;
use crate::models::{
	Shape, ShapeSettings, ShapeType, TextureLayout, UvAngle, UvFace, UvMirror, UvOffset, Vector3,
};
use crate::renderer::RenderableFace;
use glam::{Mat4, Vec3};

/// Minecraft skin texture format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinFormat {
	/// Classic 64x32 skin (pre-1.8)
	Classic,
	/// Modern 64x64 skin (1.8+)
	Modern,
}

impl SkinFormat {
	pub fn from_dimensions(width: u32, height: u32) -> Option<Self> {
		match (width, height) {
			(64, 32) => Some(SkinFormat::Classic),
			(64, 64) => Some(SkinFormat::Modern),
			_ => None,
		}
	}
}

/// Minecraft arm model variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmModel {
	/// Standard 4px wide arms (Steve)
	Regular,
	/// Slim 3px wide arms (Alex)
	Slim,
}

impl ArmModel {
	pub fn arm_width(&self) -> f32 {
		match self {
			ArmModel::Regular => 4.0,
			ArmModel::Slim => 3.0,
		}
	}
}

// ---------------------------------------------------------------------------
// UV mapping
// ---------------------------------------------------------------------------

/// UV offsets for all 6 faces of a box, in pixel coordinates.
struct BoxUv {
	front: (f32, f32),
	back: (f32, f32),
	right: (f32, f32),
	left: (f32, f32),
	top: (f32, f32),
	bottom: (f32, f32),
}

fn uv(x: f32, y: f32) -> UvFace {
	UvFace {
		offset: UvOffset { x, y },
		mirror: UvMirror {
			x: false,
			y: false,
		},
		angle: UvAngle(0),
	}
}

fn uv_mirrored(x: f32, y: f32) -> UvFace {
	UvFace {
		offset: UvOffset { x, y },
		mirror: UvMirror { x: true, y: false },
		angle: UvAngle(0),
	}
}

fn box_uv_to_layout(b: &BoxUv, _box_w: f32, _box_h: f32, _box_d: f32) -> TextureLayout {
	TextureLayout {
		front: Some(uv(b.front.0, b.front.1)),
		back: Some(uv(b.back.0, b.back.1)),
		right: Some(uv(b.right.0, b.right.1)),
		left: Some(uv(b.left.0, b.left.1)),
		top: Some(uv(b.top.0, b.top.1)),
		bottom: Some(uv(b.bottom.0, b.bottom.1)),
	}
}

/// Build a mirrored UV layout for Classic skins (right side = flipped left side).
fn box_uv_to_layout_mirrored(b: &BoxUv, _box_w: f32, _box_h: f32, _box_d: f32) -> TextureLayout {
	TextureLayout {
		front: Some(uv_mirrored(b.front.0, b.front.1)),
		back: Some(uv_mirrored(b.back.0, b.back.1)),
		right: Some(uv(b.left.0, b.left.1)),
		left: Some(uv(b.right.0, b.right.1)),
		top: Some(uv_mirrored(b.top.0, b.top.1)),
		bottom: Some(uv_mirrored(b.bottom.0, b.bottom.1)),
	}
}

// -- Modern skin (64x64) UV constants --

const HEAD_UV: BoxUv = BoxUv {
	front: (8.0, 8.0),
	back: (24.0, 8.0),
	right: (0.0, 8.0),
	left: (16.0, 8.0),
	top: (8.0, 0.0),
	bottom: (16.0, 0.0),
};

const HEAD_OVERLAY_UV: BoxUv = BoxUv {
	front: (40.0, 8.0),
	back: (56.0, 8.0),
	right: (32.0, 8.0),
	left: (48.0, 8.0),
	top: (40.0, 0.0),
	bottom: (48.0, 0.0),
};

const BODY_UV: BoxUv = BoxUv {
	front: (20.0, 20.0),
	back: (32.0, 20.0),
	right: (16.0, 20.0),
	left: (28.0, 20.0),
	top: (20.0, 16.0),
	bottom: (28.0, 16.0),
};

const BODY_OVERLAY_UV: BoxUv = BoxUv {
	front: (20.0, 36.0),
	back: (32.0, 36.0),
	right: (16.0, 36.0),
	left: (28.0, 36.0),
	top: (20.0, 32.0),
	bottom: (28.0, 32.0),
};

// "Right arm" in Minecraft terminology = player's right arm (viewer's left).
// Located at x=40, y=16 area in the texture.
const RIGHT_ARM_UV: BoxUv = BoxUv {
	front: (44.0, 20.0),
	back: (52.0, 20.0),
	right: (40.0, 20.0),
	left: (48.0, 20.0),
	top: (44.0, 16.0),
	bottom: (48.0, 16.0),
};

const RIGHT_ARM_OVERLAY_UV: BoxUv = BoxUv {
	front: (44.0, 36.0),
	back: (52.0, 36.0),
	right: (40.0, 36.0),
	left: (48.0, 36.0),
	top: (44.0, 32.0),
	bottom: (48.0, 32.0),
};

// "Left arm" in Minecraft terminology = player's left arm (viewer's right).
// Located at x=32, y=48 area in the modern texture.
const LEFT_ARM_UV: BoxUv = BoxUv {
	front: (36.0, 52.0),
	back: (44.0, 52.0),
	right: (32.0, 52.0),
	left: (40.0, 52.0),
	top: (36.0, 48.0),
	bottom: (40.0, 48.0),
};

const LEFT_ARM_OVERLAY_UV: BoxUv = BoxUv {
	front: (52.0, 52.0),
	back: (60.0, 52.0),
	right: (48.0, 52.0),
	left: (56.0, 52.0),
	top: (52.0, 48.0),
	bottom: (56.0, 48.0),
};

// "Right leg" in Minecraft terminology = player's right leg (viewer's left).
// Located at x=0, y=16 area.
const RIGHT_LEG_UV: BoxUv = BoxUv {
	front: (4.0, 20.0),
	back: (12.0, 20.0),
	right: (0.0, 20.0),
	left: (8.0, 20.0),
	top: (4.0, 16.0),
	bottom: (8.0, 16.0),
};

const RIGHT_LEG_OVERLAY_UV: BoxUv = BoxUv {
	front: (4.0, 36.0),
	back: (12.0, 36.0),
	right: (0.0, 36.0),
	left: (8.0, 36.0),
	top: (4.0, 32.0),
	bottom: (8.0, 32.0),
};

// "Left leg" in Minecraft terminology = player's left leg (viewer's right).
// Located at x=16, y=48 area in the modern texture.
const LEFT_LEG_UV: BoxUv = BoxUv {
	front: (20.0, 52.0),
	back: (28.0, 52.0),
	right: (16.0, 52.0),
	left: (24.0, 52.0),
	top: (20.0, 48.0),
	bottom: (24.0, 48.0),
};

const LEFT_LEG_OVERLAY_UV: BoxUv = BoxUv {
	front: (4.0, 52.0),
	back: (12.0, 52.0),
	right: (0.0, 52.0),
	left: (8.0, 52.0),
	top: (4.0, 48.0),
	bottom: (8.0, 48.0),
};

// ---------------------------------------------------------------------------
// Shape builder
// ---------------------------------------------------------------------------

/// Scale factor to match Hytale character coordinate space.
/// Hytale body ~127 units tall, Minecraft ~32 units → 4x scale.
const MC_SCALE: f32 = 4.0;

fn mc_shape(size: Vector3, texture_layout: TextureLayout) -> Shape {
	Shape {
		offset: Vector3::zero(),
		stretch: Vector3 {
			x: 1.0,
			y: 1.0,
			z: 1.0,
		},
		texture_layout,
		shape_type: ShapeType::Box,
		settings: ShapeSettings {
			size: Some(size),
			normal: None,
			is_piece: None,
			is_static_box: None,
		},
		unwrap_mode: "custom".to_string(),
		visible: true,
		double_sided: false,
		shading_mode: "flat".to_string(),
	}
}

/// Create an overlay shape — same as base but 1 unit larger in each dimension
fn mc_overlay_shape(base_size: Vector3, texture_layout: TextureLayout) -> Shape {
	mc_shape(
		Vector3 {
			x: base_size.x + 1.0,
			y: base_size.y + 1.0,
			z: base_size.z + 1.0,
		},
		texture_layout,
	)
}

// ---------------------------------------------------------------------------
// Skeleton
// ---------------------------------------------------------------------------

/// Body part with its shape and world-space transform
struct BodyPart {
	name: &'static str,
	shape: Shape,
	transform: Mat4,
}

fn build_skeleton(
	format: SkinFormat,
	arm_model: ArmModel,
	include_overlay: bool,
) -> Vec<BodyPart> {
	let arm_w = arm_model.arm_width();

	// Skeleton positions (in MC pixel units, before scaling).
	// Origin is at the feet. Y-up.
	//
	// Convention: character RIGHT = +X (matching Hytale renderer).
	// In a front-facing view, +X appears on the viewer's RIGHT.
	//
	// Total height: legs(12) + body(12) + head(8) = 32
	// Body parts are positioned at their center.
	// Shift down by 1 unit so the top of the head doesn't clip the camera edge.
	let y_offset = -1.0;
	let head_y = 12.0 + 12.0 + 4.0 + y_offset; // legs + body + half head
	let body_y = 12.0 + 6.0 + y_offset; // legs + half body
	let arm_y = 12.0 + 6.0 + y_offset; // legs + half arm (arms hang from shoulders)
	let leg_y = 6.0 + y_offset; // half leg height

	// Horizontal offsets
	let arm_x = 4.0 + arm_w / 2.0; // half body width + half arm width
	let leg_x = 2.0; // half of half body width

	let scale = Mat4::from_scale(Vec3::splat(MC_SCALE));

	// Head
	let head_size = Vector3 {
		x: 8.0,
		y: 8.0,
		z: 8.0,
	};
	// Body
	let body_size = Vector3 {
		x: 8.0,
		y: 12.0,
		z: 4.0,
	};
	// Arm
	let arm_size = Vector3 {
		x: arm_w,
		y: 12.0,
		z: 4.0,
	};
	// Leg
	let leg_size = Vector3 {
		x: 4.0,
		y: 12.0,
		z: 4.0,
	};

	let is_classic = format == SkinFormat::Classic;

	let mut parts = Vec::new();

	// --- Base layer ---
	parts.push(BodyPart {
		name: "mc_head",
		shape: mc_shape(head_size, box_uv_to_layout(&HEAD_UV, 8.0, 8.0, 8.0)),
		transform: scale * Mat4::from_translation(Vec3::new(0.0, head_y, 0.0)),
	});
	parts.push(BodyPart {
		name: "mc_body",
		shape: mc_shape(body_size, box_uv_to_layout(&BODY_UV, 8.0, 12.0, 4.0)),
		transform: scale * Mat4::from_translation(Vec3::new(0.0, body_y, 0.0)),
	});
	// Right arm/leg at +X (viewer's right in front view, matching Hytale convention)
	parts.push(BodyPart {
		name: "mc_right_arm",
		shape: mc_shape(arm_size, box_uv_to_layout(&RIGHT_ARM_UV, arm_w, 12.0, 4.0)),
		transform: scale * Mat4::from_translation(Vec3::new(arm_x, arm_y, 0.0)),
	});
	parts.push(BodyPart {
		name: "mc_left_arm",
		shape: mc_shape(
			arm_size,
			if is_classic {
				box_uv_to_layout_mirrored(&RIGHT_ARM_UV, arm_w, 12.0, 4.0)
			} else {
				box_uv_to_layout(&LEFT_ARM_UV, arm_w, 12.0, 4.0)
			},
		),
		transform: scale * Mat4::from_translation(Vec3::new(-arm_x, arm_y, 0.0)),
	});
	parts.push(BodyPart {
		name: "mc_right_leg",
		shape: mc_shape(leg_size, box_uv_to_layout(&RIGHT_LEG_UV, 4.0, 12.0, 4.0)),
		transform: scale * Mat4::from_translation(Vec3::new(leg_x, leg_y, 0.0)),
	});
	parts.push(BodyPart {
		name: "mc_left_leg",
		shape: mc_shape(
			leg_size,
			if is_classic {
				box_uv_to_layout_mirrored(&RIGHT_LEG_UV, 4.0, 12.0, 4.0)
			} else {
				box_uv_to_layout(&LEFT_LEG_UV, 4.0, 12.0, 4.0)
			},
		),
		transform: scale * Mat4::from_translation(Vec3::new(-leg_x, leg_y, 0.0)),
	});

	// --- Overlay layer ---
	if include_overlay {
		parts.push(BodyPart {
			name: "mc_head_overlay",
			shape: mc_overlay_shape(head_size, box_uv_to_layout(&HEAD_OVERLAY_UV, 8.0, 8.0, 8.0)),
			transform: scale * Mat4::from_translation(Vec3::new(0.0, head_y, 0.0)),
		});

		if !is_classic {
			parts.push(BodyPart {
				name: "mc_body_overlay",
				shape: mc_overlay_shape(
					body_size,
					box_uv_to_layout(&BODY_OVERLAY_UV, 8.0, 12.0, 4.0),
				),
				transform: scale * Mat4::from_translation(Vec3::new(0.0, body_y, 0.0)),
			});
			parts.push(BodyPart {
				name: "mc_right_arm_overlay",
				shape: mc_overlay_shape(
					arm_size,
					box_uv_to_layout(&RIGHT_ARM_OVERLAY_UV, arm_w, 12.0, 4.0),
				),
				transform: scale * Mat4::from_translation(Vec3::new(arm_x, arm_y, 0.0)),
			});
			parts.push(BodyPart {
				name: "mc_left_arm_overlay",
				shape: mc_overlay_shape(
					arm_size,
					box_uv_to_layout(&LEFT_ARM_OVERLAY_UV, arm_w, 12.0, 4.0),
				),
				transform: scale * Mat4::from_translation(Vec3::new(-arm_x, arm_y, 0.0)),
			});
			parts.push(BodyPart {
				name: "mc_right_leg_overlay",
				shape: mc_overlay_shape(
					leg_size,
					box_uv_to_layout(&RIGHT_LEG_OVERLAY_UV, 4.0, 12.0, 4.0),
				),
				transform: scale * Mat4::from_translation(Vec3::new(leg_x, leg_y, 0.0)),
			});
			parts.push(BodyPart {
				name: "mc_left_leg_overlay",
				shape: mc_overlay_shape(
					leg_size,
					box_uv_to_layout(&LEFT_LEG_OVERLAY_UV, 4.0, 12.0, 4.0),
				),
				transform: scale * Mat4::from_translation(Vec3::new(-leg_x, leg_y, 0.0)),
			});
		}
	}

	parts
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build renderable 3D faces for only the Minecraft head (+ overlay if included).
///
/// Used for the `cube` / isometric head view.
pub fn build_minecraft_head_faces(
	format: SkinFormat,
	include_overlay: bool,
) -> Vec<RenderableFace> {
	let parts = build_skeleton(format, ArmModel::Regular, include_overlay);
	let mut faces = Vec::new();

	for part in parts
		.iter()
		.filter(|p| p.name == "mc_head" || p.name == "mc_head_overlay")
	{
		let geometry = geometry::generate_geometry(&part.shape, part.transform);
		for face in geometry {
			faces.push(RenderableFace {
				face,
				transform: part.transform,
				shape: Some(part.shape.clone()),
				node_name: Some(part.name.to_string()),
				texture: None,
				tint: None,
			});
		}
	}

	faces
}

/// Build renderable 3D faces from a Minecraft skin texture.
///
/// The returned faces can be passed directly to `renderer::render_scene_tinted`
/// along with the skin `Texture` and a camera preset.
pub fn build_minecraft_faces(
	format: SkinFormat,
	arm_model: ArmModel,
	include_overlay: bool,
) -> Vec<RenderableFace> {
	let parts = build_skeleton(format, arm_model, include_overlay);
	let mut faces = Vec::new();

	for part in &parts {
		let geometry = geometry::generate_geometry(&part.shape, part.transform);
		for face in geometry {
			faces.push(RenderableFace {
				face,
				transform: part.transform,
				shape: Some(part.shape.clone()),
				node_name: Some(part.name.to_string()),
				texture: None,
				tint: None,
			});
		}
	}

	faces
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_skin_format_detection() {
		assert_eq!(
			SkinFormat::from_dimensions(64, 64),
			Some(SkinFormat::Modern)
		);
		assert_eq!(
			SkinFormat::from_dimensions(64, 32),
			Some(SkinFormat::Classic)
		);
		assert_eq!(SkinFormat::from_dimensions(128, 128), None);
	}

	#[test]
	fn test_modern_base_face_count() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);
		// 6 body parts * 6 faces each = 36
		assert_eq!(faces.len(), 36);
	}

	#[test]
	fn test_modern_overlay_face_count() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, true);
		// 6 base parts * 6 faces + 6 overlay parts * 6 faces = 72
		assert_eq!(faces.len(), 72);
	}

	#[test]
	fn test_classic_base_face_count() {
		let faces = build_minecraft_faces(SkinFormat::Classic, ArmModel::Regular, false);
		// 6 body parts * 6 faces each = 36
		assert_eq!(faces.len(), 36);
	}

	#[test]
	fn test_classic_overlay_face_count() {
		let faces = build_minecraft_faces(SkinFormat::Classic, ArmModel::Regular, true);
		// Classic: only head has overlay = 6 base parts * 6 faces + 1 overlay * 6 faces = 42
		assert_eq!(faces.len(), 42);
	}

	#[test]
	fn test_slim_arm_width() {
		assert_eq!(ArmModel::Slim.arm_width(), 3.0);
		assert_eq!(ArmModel::Regular.arm_width(), 4.0);
	}

	#[test]
	fn test_slim_produces_valid_faces() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Slim, false);
		assert_eq!(faces.len(), 36);

		// Each face should have 4 vertices
		for f in &faces {
			assert_eq!(f.face.vertices.len(), 4);
		}
	}

	#[test]
	fn test_all_faces_have_shape() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, true);
		for f in &faces {
			assert!(f.shape.is_some());
			assert!(f.node_name.is_some());
		}
	}

	#[test]
	fn test_head_uv_offsets() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);
		let head_front = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_head") && f.face.texture_face == "front")
			.expect("Should have mc_head front face");

		let shape = head_front.shape.as_ref().unwrap();
		let front_uv = shape.texture_layout.front.as_ref().unwrap();
		assert_eq!(front_uv.offset.x, 8.0);
		assert_eq!(front_uv.offset.y, 8.0);
		assert!(!front_uv.mirror.x);
	}

	#[test]
	fn test_overlay_is_larger_than_base() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, true);

		let head_base = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_head"))
			.unwrap();
		let head_overlay = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_head_overlay"))
			.unwrap();

		let base_size = head_base.shape.as_ref().unwrap().settings.size.unwrap();
		let overlay_size = head_overlay
			.shape
			.as_ref()
			.unwrap()
			.settings
			.size
			.unwrap();

		assert!(overlay_size.x > base_size.x);
		assert!(overlay_size.y > base_size.y);
		assert!(overlay_size.z > base_size.z);
	}

	#[test]
	fn test_classic_mirrored_arms() {
		let faces = build_minecraft_faces(SkinFormat::Classic, ArmModel::Regular, false);

		let left_arm_front = faces
			.iter()
			.find(|f| {
				f.node_name.as_deref() == Some("mc_left_arm") && f.face.texture_face == "front"
			})
			.unwrap();

		let shape = left_arm_front.shape.as_ref().unwrap();
		let front_uv = shape.texture_layout.front.as_ref().unwrap();
		// Classic: left arm mirrors right arm's front face
		assert!(front_uv.mirror.x);
		assert_eq!(front_uv.offset.x, RIGHT_ARM_UV.front.0);
		assert_eq!(front_uv.offset.y, RIGHT_ARM_UV.front.1);
	}

	#[test]
	fn test_scale_factor_applied() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);

		// Head center should be at Y = 27 * MC_SCALE = 108 (shifted down 1 unit)
		let head_face = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_head"))
			.unwrap();

		// The transform should include the scale factor
		let translation = head_face.transform.col(3);
		assert!(
			(translation.y - 27.0 * MC_SCALE).abs() < 0.1,
			"Head Y should be {} but was {}",
			27.0 * MC_SCALE,
			translation.y
		);
	}

	#[test]
	fn test_right_arm_at_positive_x() {
		// Convention: character right = +X (matching Hytale renderer)
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);

		let right_arm = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_right_arm"))
			.unwrap();
		let left_arm = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_left_arm"))
			.unwrap();

		// Right arm should be at positive X
		let right_x = right_arm.transform.col(3).x;
		let left_x = left_arm.transform.col(3).x;
		assert!(
			right_x > 0.0,
			"Right arm should be at +X, was {}",
			right_x
		);
		assert!(left_x < 0.0, "Left arm should be at -X, was {}", left_x);
	}

	#[test]
	fn test_arm_y_aligns_with_body() {
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);

		let body = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_body"))
			.unwrap();
		let arm = faces
			.iter()
			.find(|f| f.node_name.as_deref() == Some("mc_right_arm"))
			.unwrap();

		// Arm center Y should equal body center Y (both 17 * scale = 68)
		let body_y = body.transform.col(3).y;
		let arm_y = arm.transform.col(3).y;
		assert!(
			(body_y - arm_y).abs() < 0.1,
			"Arm Y ({}) should match body Y ({})",
			arm_y,
			body_y
		);
	}

	/// Integration test: render a diagnostic skin with unique colors per face
	/// and verify the front-facing view shows the correct textures.
	#[test]
	fn test_render_front_view_face_colors() {
		use crate::{camera, renderer, texture};

		// Create a 64x64 diagnostic skin texture where each body part's
		// front face has a unique color and back face has a different one.
		let mut img = image::RgbaImage::new(64, 64);

		// Fill with transparent
		for px in img.pixels_mut() {
			*px = image::Rgba([0, 0, 0, 0]);
		}

		// Helper to fill a rect
		let fill = |img: &mut image::RgbaImage, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]| {
			for dy in 0..h {
				for dx in 0..w {
					if x + dx < 64 && y + dy < 64 {
						img.put_pixel(x + dx, y + dy, image::Rgba(color));
					}
				}
			}
		};

		let red = [255, 0, 0, 255]; // front faces
		let green = [0, 255, 0, 255]; // back faces
		let blue = [0, 0, 255, 255]; // right faces
		let yellow = [255, 255, 0, 255]; // left faces
		let cyan = [0, 255, 255, 255]; // top faces
		let magenta = [255, 0, 255, 255]; // bottom faces

		// HEAD (8x8x8) - fill all 6 face regions
		fill(&mut img, 8, 8, 8, 8, red); // front
		fill(&mut img, 24, 8, 8, 8, green); // back
		fill(&mut img, 0, 8, 8, 8, blue); // right
		fill(&mut img, 16, 8, 8, 8, yellow); // left
		fill(&mut img, 8, 0, 8, 8, cyan); // top
		fill(&mut img, 16, 0, 8, 8, magenta); // bottom

		// BODY (8x12x4) - front and back in different colors
		fill(&mut img, 20, 20, 8, 12, red); // front
		fill(&mut img, 32, 20, 8, 12, green); // back
		fill(&mut img, 16, 20, 4, 12, blue); // right
		fill(&mut img, 28, 20, 4, 12, yellow); // left
		fill(&mut img, 20, 16, 8, 4, cyan); // top
		fill(&mut img, 28, 16, 8, 4, magenta); // bottom

		// RIGHT ARM (4x12x4)
		fill(&mut img, 44, 20, 4, 12, red); // front
		fill(&mut img, 52, 20, 4, 12, green); // back
		fill(&mut img, 40, 20, 4, 12, blue); // right
		fill(&mut img, 48, 20, 4, 12, yellow); // left
		fill(&mut img, 44, 16, 4, 4, cyan); // top
		fill(&mut img, 48, 16, 4, 4, magenta); // bottom

		// LEFT ARM (4x12x4)
		fill(&mut img, 36, 52, 4, 12, red); // front
		fill(&mut img, 44, 52, 4, 12, green); // back
		fill(&mut img, 32, 52, 4, 12, blue); // right
		fill(&mut img, 40, 52, 4, 12, yellow); // left
		fill(&mut img, 36, 48, 4, 4, cyan); // top
		fill(&mut img, 40, 48, 4, 4, magenta); // bottom

		// RIGHT LEG (4x12x4)
		fill(&mut img, 4, 20, 4, 12, red); // front
		fill(&mut img, 12, 20, 4, 12, green); // back
		fill(&mut img, 0, 20, 4, 12, blue); // right
		fill(&mut img, 8, 20, 4, 12, yellow); // left
		fill(&mut img, 4, 16, 4, 4, cyan); // top
		fill(&mut img, 8, 16, 4, 4, magenta); // bottom

		// LEFT LEG (4x12x4)
		fill(&mut img, 20, 52, 4, 12, red); // front
		fill(&mut img, 28, 52, 4, 12, green); // back
		fill(&mut img, 16, 52, 4, 12, blue); // right
		fill(&mut img, 24, 52, 4, 12, yellow); // left
		fill(&mut img, 20, 48, 4, 4, cyan); // top
		fill(&mut img, 24, 48, 4, 4, magenta); // bottom

		let tex = texture::Texture::from_image(image::DynamicImage::ImageRgba8(img));

		// Build faces and render with full_body_front camera
		let faces = build_minecraft_faces(SkinFormat::Modern, ArmModel::Regular, false);
		let cam = camera::Camera::full_body_front();
		let tint = renderer::TintConfig::default();

		let output =
			renderer::render_scene_tinted(&faces, &tex, &cam, 180, 360, &tint).unwrap();

		// In a perfect front orthographic view, ONLY the front faces (+Z normal)
		// should be visible. Every visible pixel should be RED (our front face color).
		//
		// Sample the center of each body part in the output.
		// Character height = 128 units, centered at Y=64.
		// Camera: ortho_size=130, center at Y=63.5.
		// Output: 180x360.

		// Helper to check a pixel is approximately the expected color (allow rounding)
		let check_pixel = |x: u32, y: u32, expected: [u8; 3], label: &str| {
			let pixel = output.get_pixel(x, y);
			let actual = [pixel[0], pixel[1], pixel[2]];
			let close = actual[0].abs_diff(expected[0]) <= 5
				&& actual[1].abs_diff(expected[1]) <= 5
				&& actual[2].abs_diff(expected[2]) <= 5;
			assert!(
				close,
				"{} at ({},{}) should be ~{:?} but was {:?}",
				label, x, y, expected, actual
			);
		};

		// The character spans roughly the full height of the output.
		// Body center is at world Y=72 (18*4), mapped to screen.
		// Screen center Y ≈ 180 (out of 360) for Y=63.5
		// Let's just check that the center of the output (where the body is) is RED
		let center_x = 90; // middle of 180px wide
		let center_y = 180; // middle of 360px tall (roughly body center)

		let body_pixel = output.get_pixel(center_x, center_y);
		assert!(
			body_pixel[3] > 0,
			"Body center pixel should not be transparent"
		);

		// The body front face should be RED
		check_pixel(center_x, center_y, [255, 0, 0], "Body front");

		// Check right arm (at +X, viewer's right) - should be RED
		let right_arm_x = center_x + 40; // roughly where the right arm is
		let right_arm_pixel = output.get_pixel(right_arm_x, center_y);
		if right_arm_pixel[3] > 0 {
			check_pixel(right_arm_x, center_y, [255, 0, 0], "Right arm front");
		}

		// Check left arm (at -X, viewer's left) - should also be RED
		let left_arm_x = center_x - 40;
		let left_arm_pixel = output.get_pixel(left_arm_x, center_y);
		if left_arm_pixel[3] > 0 {
			check_pixel(left_arm_x, center_y, [255, 0, 0], "Left arm front");
		}

		// Save diagnostic output for visual inspection
		output.save("/tmp/mc_diagnostic_front.png").unwrap();

		// Also render front_right view to check side faces
		let cam_fr = camera::Camera::front_right_view();
		let output_fr =
			renderer::render_scene_tinted(&faces, &tex, &cam_fr, 180, 360, &tint).unwrap();
		output_fr.save("/tmp/mc_diagnostic_front_right.png").unwrap();
	}
}
