//! Animation sampling and pose data
//!
//! This module provides functionality for sampling animation keyframes at specific
//! times and generating pose data that can be applied to a scene graph.

use crate::models::{
	BlockyAnimation, InterpolationType, NodeAnimation, OrientationKeyframe, PositionKeyframe,
	Quaternion, Vector3,
};
use std::collections::HashMap;

/// Pose data for a single node at a specific frame
#[derive(Debug, Clone, Default)]
pub struct NodePose {
	/// Position delta to add to the bind pose position
	pub position_delta: Option<Vector3>,
	/// Orientation delta to multiply with the bind pose orientation
	pub orientation_delta: Option<Quaternion>,
}

/// Pose data for all animated nodes at a specific frame
#[derive(Debug, Clone, Default)]
pub struct AnimationPose {
	/// Pose data keyed by node name
	pub node_poses: HashMap<String, NodePose>,
}

impl AnimationPose {
	/// Create a new empty pose
	pub fn new() -> Self {
		Self {
			node_poses: HashMap::new(),
		}
	}

	/// Get pose for a specific node
	pub fn get(&self, node_name: &str) -> Option<&NodePose> {
		self.node_poses.get(node_name)
	}
}

/// Sample an animation at a specific frame to get pose data
pub fn sample_animation(animation: &BlockyAnimation, frame: f32) -> AnimationPose {
	let mut pose = AnimationPose::new();
	let duration = animation.duration as f32;

	// Wrap frame for looping
	let wrapped_frame = if animation.duration > 0 {
		frame % duration
	} else {
		0.0
	};

	for (node_name, node_anim) in &animation.node_animations {
		let node_pose = sample_node_animation(node_anim, wrapped_frame, duration);

		if node_pose.position_delta.is_some() || node_pose.orientation_delta.is_some() {
			pose.node_poses.insert(node_name.clone(), node_pose);
		}
	}

	pose
}

fn sample_node_animation(node_anim: &NodeAnimation, frame: f32, duration: f32) -> NodePose {
	NodePose {
		position_delta: sample_position_keyframes(&node_anim.position, frame, duration),
		orientation_delta: sample_orientation_keyframes(&node_anim.orientation, frame, duration),
	}
}

fn sample_position_keyframes(
	keyframes: &[PositionKeyframe],
	frame: f32,
	duration: f32,
) -> Option<Vector3> {
	if keyframes.is_empty() {
		return None;
	}

	let (before, after, is_wrapped) =
		find_surrounding_keyframes_with_wrap(keyframes, frame, duration, |kf| kf.time as f32);

	match (before, after) {
		(Some(kf), None) | (None, Some(kf)) => Some(kf.delta),
		(Some(kf1), Some(kf2)) => {
			let t = calculate_interpolation_factor_wrapped(
				kf1.time as f32,
				kf2.time as f32,
				frame,
				duration,
				is_wrapped,
			);
			let t = apply_interpolation_curve(t, kf2.interpolation_type);
			Some(lerp_vector3(&kf1.delta, &kf2.delta, t))
		}
		(None, None) => None,
	}
}

fn sample_orientation_keyframes(
	keyframes: &[OrientationKeyframe],
	frame: f32,
	duration: f32,
) -> Option<Quaternion> {
	if keyframes.is_empty() {
		return None;
	}

	let (before, after, is_wrapped) =
		find_surrounding_keyframes_with_wrap(keyframes, frame, duration, |kf| kf.time as f32);

	match (before, after) {
		(Some(kf), None) | (None, Some(kf)) => Some(kf.delta),
		(Some(kf1), Some(kf2)) => {
			let t = calculate_interpolation_factor_wrapped(
				kf1.time as f32,
				kf2.time as f32,
				frame,
				duration,
				is_wrapped,
			);
			let t = apply_interpolation_curve(t, kf2.interpolation_type);
			Some(slerp_quaternion(&kf1.delta, &kf2.delta, t))
		}
		(None, None) => None,
	}
}

/// Find keyframes before and after target time, supporting wrap-around
fn find_surrounding_keyframes_with_wrap<T, F>(
	keyframes: &[T],
	time: f32,
	_duration: f32,
	get_time: F,
) -> (Option<&T>, Option<&T>, bool)
where
	F: Fn(&T) -> f32,
{
	if keyframes.is_empty() {
		return (None, None, false);
	}

	if keyframes.len() == 1 {
		return (Some(&keyframes[0]), None, false);
	}

	let mut before: Option<&T> = None;
	let mut after: Option<&T> = None;

	let first_kf = keyframes
		.iter()
		.min_by(|a, b| get_time(a).partial_cmp(&get_time(b)).unwrap());
	let last_kf = keyframes
		.iter()
		.max_by(|a, b| get_time(a).partial_cmp(&get_time(b)).unwrap());

	let first_time = first_kf.map(|k| get_time(k)).unwrap_or(0.0);
	let last_time = last_kf.map(|k| get_time(k)).unwrap_or(0.0);

	// Handle wrap-around regions
	if time < first_time {
		return (last_kf, first_kf, true);
	}
	if time > last_time {
		return (last_kf, first_kf, true);
	}

	for kf in keyframes {
		let kf_time = get_time(kf);

		if kf_time <= time {
			if before.is_none() || kf_time > get_time(before.unwrap()) {
				before = Some(kf);
			}
		}

		if kf_time > time {
			if after.is_none() || kf_time < get_time(after.unwrap()) {
				after = Some(kf);
			}
		}
	}

	if before.is_some() && after.is_none() {
		return (last_kf, first_kf, true);
	}
	if after.is_some() && before.is_none() {
		return (last_kf, first_kf, true);
	}

	(before, after, false)
}

/// Calculate interpolation factor between keyframes, handling wrap-around
fn calculate_interpolation_factor_wrapped(
	t1: f32,
	t2: f32,
	current: f32,
	duration: f32,
	is_wrapped: bool,
) -> f32 {
	if is_wrapped {
		let effective_t2 = t2 + duration;
		let effective_current = if current < t1 {
			current + duration
		} else {
			current
		};

		let range = effective_t2 - t1;
		if range.abs() < 0.0001 {
			return 0.0;
		}
		((effective_current - t1) / range).clamp(0.0, 1.0)
	} else {
		if (t2 - t1).abs() < 0.0001 {
			return 0.0;
		}
		((current - t1) / (t2 - t1)).clamp(0.0, 1.0)
	}
}

fn apply_interpolation_curve(t: f32, interp_type: InterpolationType) -> f32 {
	match interp_type {
		InterpolationType::Step => 0.0,
		InterpolationType::Linear => t,
		InterpolationType::Smooth => smoothstep(t),
	}
}

/// Smoothstep function for eased interpolation
fn smoothstep(t: f32) -> f32 {
	t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation between two Vector3 values
fn lerp_vector3(a: &Vector3, b: &Vector3, t: f32) -> Vector3 {
	Vector3 {
		x: a.x + (b.x - a.x) * t,
		y: a.y + (b.y - a.y) * t,
		z: a.z + (b.z - a.z) * t,
	}
}

/// Spherical linear interpolation between two quaternions
fn slerp_quaternion(a: &Quaternion, b: &Quaternion, t: f32) -> Quaternion {
	// Calculate dot product
	let mut dot = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;

	// If negative dot, negate one quaternion to take shorter path
	let mut b_adj = *b;
	if dot < 0.0 {
		b_adj.x = -b_adj.x;
		b_adj.y = -b_adj.y;
		b_adj.z = -b_adj.z;
		b_adj.w = -b_adj.w;
		dot = -dot;
	}

	// Clamp dot to valid range
	dot = dot.clamp(-1.0, 1.0);

	// If quaternions are very close, use linear interpolation
	if dot > 0.9995 {
		return normalize_quaternion(&Quaternion {
			x: a.x + (b_adj.x - a.x) * t,
			y: a.y + (b_adj.y - a.y) * t,
			z: a.z + (b_adj.z - a.z) * t,
			w: a.w + (b_adj.w - a.w) * t,
		});
	}

	// Calculate slerp
	let theta = dot.acos();
	let sin_theta = theta.sin();

	if sin_theta.abs() < 0.0001 {
		// Quaternions are nearly parallel, use linear interpolation
		return normalize_quaternion(&Quaternion {
			x: a.x + (b_adj.x - a.x) * t,
			y: a.y + (b_adj.y - a.y) * t,
			z: a.z + (b_adj.z - a.z) * t,
			w: a.w + (b_adj.w - a.w) * t,
		});
	}

	let factor_a = ((1.0 - t) * theta).sin() / sin_theta;
	let factor_b = (t * theta).sin() / sin_theta;

	Quaternion {
		x: a.x * factor_a + b_adj.x * factor_b,
		y: a.y * factor_a + b_adj.y * factor_b,
		z: a.z * factor_a + b_adj.z * factor_b,
		w: a.w * factor_a + b_adj.w * factor_b,
	}
}

/// Normalize a quaternion to unit length
fn normalize_quaternion(q: &Quaternion) -> Quaternion {
	let len = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
	if len < 0.0001 {
		return Quaternion::identity();
	}
	Quaternion {
		x: q.x / len,
		y: q.y / len,
		z: q.z / len,
		w: q.w / len,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::parse_blockyanim;

	#[test]
	fn test_sample_empty_animation() {
		let json = r#"{ "duration": 60, "nodeAnimations": {} }"#;
		let anim = parse_blockyanim(json).unwrap();
		let pose = sample_animation(&anim, 0.0);
		assert!(pose.node_poses.is_empty());
	}

	#[test]
	fn test_sample_single_position_keyframe() {
		let json = r#"
        {
            "duration": 60,
            "nodeAnimations": {
                "Test": {
                    "position": [{ "time": 0, "delta": { "x": 1, "y": 2, "z": 3 }, "interpolationType": "smooth" }],
                    "orientation": [],
                    "shapeStretch": [],
                    "shapeVisible": [],
                    "shapeUvOffset": []
                }
            }
        }
        "#;
		let anim = parse_blockyanim(json).unwrap();

		// Should return the single keyframe's value at any time
		let pose = sample_animation(&anim, 0.0);
		let node_pose = pose.get("Test").unwrap();
		let pos = node_pose.position_delta.unwrap();
		assert_eq!(pos.x, 1.0);
		assert_eq!(pos.y, 2.0);
		assert_eq!(pos.z, 3.0);
	}

	#[test]
	fn test_sample_position_interpolation() {
		let json = r#"
        {
            "duration": 60,
            "nodeAnimations": {
                "Test": {
                    "position": [
                        { "time": 0, "delta": { "x": 0, "y": 0, "z": 0 }, "interpolationType": "smooth" },
                        { "time": 60, "delta": { "x": 10, "y": 20, "z": 30 }, "interpolationType": "linear" }
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

		// At frame 30 (halfway), with linear interpolation, should be at midpoint
		let pose = sample_animation(&anim, 30.0);
		let node_pose = pose.get("Test").unwrap();
		let pos = node_pose.position_delta.unwrap();
		assert!((pos.x - 5.0).abs() < 0.1);
		assert!((pos.y - 10.0).abs() < 0.1);
		assert!((pos.z - 15.0).abs() < 0.1);
	}

	#[test]
	fn test_sample_orientation_keyframe() {
		let json = r#"
        {
            "duration": 60,
            "nodeAnimations": {
                "Test": {
                    "position": [],
                    "orientation": [{ "time": 0, "delta": { "x": 0, "y": 0, "z": 0, "w": 1 }, "interpolationType": "smooth" }],
                    "shapeStretch": [],
                    "shapeVisible": [],
                    "shapeUvOffset": []
                }
            }
        }
        "#;
		let anim = parse_blockyanim(json).unwrap();

		let pose = sample_animation(&anim, 0.0);
		let node_pose = pose.get("Test").unwrap();
		let orient = node_pose.orientation_delta.unwrap();
		assert_eq!(orient.w, 1.0);
		assert_eq!(orient.x, 0.0);
	}

	#[test]
	fn test_smoothstep() {
		// At t=0, should be 0
		assert!((smoothstep(0.0) - 0.0).abs() < 0.001);
		// At t=1, should be 1
		assert!((smoothstep(1.0) - 1.0).abs() < 0.001);
		// At t=0.5, should be 0.5 (smoothstep is symmetric)
		assert!((smoothstep(0.5) - 0.5).abs() < 0.001);
	}

	#[test]
	fn test_quaternion_normalize() {
		let q = Quaternion {
			x: 2.0,
			y: 0.0,
			z: 0.0,
			w: 0.0,
		};
		let normalized = normalize_quaternion(&q);
		let len = (normalized.x * normalized.x
			+ normalized.y * normalized.y
			+ normalized.z * normalized.z
			+ normalized.w * normalized.w)
			.sqrt();
		assert!((len - 1.0).abs() < 0.001);
	}

	#[test]
	fn test_sample_idle_animation_frame_0() {
		// Test with actual Idle.blockyanim file
		let path =
			std::path::Path::new("assets/Common/Characters/Animations/Default/Idle.blockyanim");
		if path.exists() {
			let anim = crate::models::parse_blockyanim_from_file(path).unwrap();
			let pose = sample_animation(&anim, 0.0);

			// Pelvis should have a position delta at frame 0
			let pelvis = pose.get("Pelvis");
			assert!(pelvis.is_some());
			let pelvis = pelvis.unwrap();
			assert!(pelvis.position_delta.is_some());

			// Check expected values from Idle.blockyanim
			let pos = pelvis.position_delta.unwrap();
			assert!((pos.y - (-0.925)).abs() < 0.01);
		}
	}
}
