use crate::geometry;
use glam::{Mat4, Vec3, Vec4};

/// Vertex with clip-space data for clipping
#[derive(Clone, Debug)]
pub struct ClipVertex {
	pub world_pos: Vec3,
	pub uv: (f32, f32),
	pub clip_pos: Vec4,
	pub normal: Vec3,
}

/// Clipping plane in clip space
#[derive(Clone, Copy, Debug)]
pub enum ClipPlane {
	Left,   // clip_x >= -clip_w
	Right,  // clip_x <= clip_w
	Bottom, // clip_y >= -clip_w
	Top,    // clip_y <= clip_w
	Near,   // clip_z >= -clip_w
	Far,    // clip_z <= clip_w
}

/// Main clipping function that clips a face against the view frustum
///
/// Returns None if the face is completely outside the frustum,
/// or Some(clipped_vertices) if any part of the face is visible.
pub fn clip_face_to_frustum(face: &geometry::Face, vp_matrix: &Mat4) -> Option<Vec<ClipVertex>> {
	// Transform all vertices to clip space
	let mut vertices: Vec<ClipVertex> = face
		.vertices
		.iter()
		.map(|v| {
			let world_pos = Vec4::from((v.position, 1.0));
			let clip_pos = *vp_matrix * world_pos;
			ClipVertex {
				world_pos: v.position,
				uv: v.uv,
				clip_pos,
				normal: v.normal,
			}
		})
		.collect();

	// Early rejection: check if all vertices are outside any single plane
	if is_trivially_rejected(&vertices) {
		return None;
	}

	// Apply Sutherland-Hodgman clipping against all 6 frustum planes
	// IMPORTANT: Near plane must be clipped FIRST to ensure w > 0 for all subsequent operations
	// This prevents division by zero or w=0 issues when geometry crosses the camera plane
	let planes = [
		ClipPlane::Near,
		ClipPlane::Left,
		ClipPlane::Right,
		ClipPlane::Bottom,
		ClipPlane::Top,
		ClipPlane::Far,
	];

	for plane in &planes {
		vertices = sutherland_hodgman_clip(vertices, *plane);
		if vertices.len() < 3 {
			return None; // Face completely clipped away
		}
	}

	Some(vertices)
}

/// Check if all vertices are outside any single plane (trivial rejection)
fn is_trivially_rejected(vertices: &[ClipVertex]) -> bool {
	let planes = [
		ClipPlane::Near,
		ClipPlane::Left,
		ClipPlane::Right,
		ClipPlane::Bottom,
		ClipPlane::Top,
		ClipPlane::Far,
	];

	for plane in &planes {
		if vertices.iter().all(|v| !is_inside(v, *plane)) {
			return true;
		}
	}

	false
}

/// Sutherland-Hodgman polygon clipping algorithm
///
/// Clips a polygon against a single plane, returning the clipped polygon.
fn sutherland_hodgman_clip(vertices: Vec<ClipVertex>, plane: ClipPlane) -> Vec<ClipVertex> {
	if vertices.len() < 3 {
		return vec![];
	}

	let mut output = Vec::new();
	let n = vertices.len();

	for i in 0..n {
		let current = &vertices[i];
		let next = &vertices[(i + 1) % n];

		let current_inside = is_inside(current, plane);
		let next_inside = is_inside(next, plane);

		match (current_inside, next_inside) {
			(true, true) => {
				// Both inside: add next vertex
				output.push(next.clone());
			}
			(true, false) => {
				// Leaving: add intersection point
				if let Some(intersection) = compute_intersection(current, next, plane) {
					output.push(intersection);
				}
			}
			(false, true) => {
				// Entering: add intersection + next vertex
				if let Some(intersection) = compute_intersection(current, next, plane) {
					output.push(intersection);
				}
				output.push(next.clone());
			}
			(false, false) => {
				// Both outside: add nothing
			}
		}
	}

	output
}

/// Check if a vertex is inside the given clipping plane
fn is_inside(vertex: &ClipVertex, plane: ClipPlane) -> bool {
	// Use signed distance directly (no epsilon buffer)
	// Epsilon is now handled in compute_intersection_t via clamping
	signed_distance(vertex.clip_pos, plane) >= 0.0
}

/// Compute the intersection point between an edge and a clipping plane
fn compute_intersection(v1: &ClipVertex, v2: &ClipVertex, plane: ClipPlane) -> Option<ClipVertex> {
	// Calculate t parameter where edge crosses plane
	let t = compute_intersection_t(v1, v2, plane)?;

	// Interpolate all attributes
	let normal = v1.normal.lerp(v2.normal, t).normalize();

	Some(ClipVertex {
		world_pos: v1.world_pos.lerp(v2.world_pos, t),
		uv: (
			v1.uv.0 + t * (v2.uv.0 - v1.uv.0),
			v1.uv.1 + t * (v2.uv.1 - v1.uv.1),
		),
		clip_pos: v1.clip_pos.lerp(v2.clip_pos, t),
		normal,
	})
}

/// Compute the t parameter for the intersection point
fn compute_intersection_t(v1: &ClipVertex, v2: &ClipVertex, plane: ClipPlane) -> Option<f32> {
	let c1 = v1.clip_pos;
	let c2 = v2.clip_pos;

	// Compute signed distances from plane
	let d1 = signed_distance(c1, plane);
	let d2 = signed_distance(c2, plane);

	let denominator = d1 - d2;

	// Use a much smaller epsilon for numerical stability
	// We need to handle edges that are nearly parallel to the plane
	if denominator.abs() < 1e-10 {
		// Edge is nearly parallel to the plane
		// If both points are very close to the plane, treat as on the plane
		// Return a midpoint interpolation
		return Some(0.5);
	}

	// t = d1 / (d1 - d2) gives intersection parameter
	let t = d1 / denominator;

	// Clamp t to [0, 1] range instead of rejecting out-of-range values
	// This handles numerical precision issues gracefully
	let t_clamped = t.clamp(0.0, 1.0);
	Some(t_clamped)
}

/// Compute signed distance from a point to a clipping plane
///
/// Positive distance means inside the plane, negative means outside.
fn signed_distance(clip_pos: Vec4, plane: ClipPlane) -> f32 {
	match plane {
		ClipPlane::Left => clip_pos.x + clip_pos.w,
		ClipPlane::Right => clip_pos.w - clip_pos.x,
		ClipPlane::Bottom => clip_pos.y + clip_pos.w,
		ClipPlane::Top => clip_pos.w - clip_pos.y,
		ClipPlane::Near => clip_pos.z + clip_pos.w,
		ClipPlane::Far => clip_pos.w - clip_pos.z,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clip_fully_inside_face() {
		// Create a simple triangle fully inside the frustum
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(0.0, 0.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(0.5, 0.0, 0.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(0.0, 0.5, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		assert!(result.is_some());

		let clipped = result.unwrap();
		assert_eq!(clipped.len(), 3); // Should still have 3 vertices
	}

	#[test]
	fn test_clip_fully_outside_face() {
		// Create a triangle completely outside the frustum (far right)
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(10.0, 0.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(11.0, 0.0, 0.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(10.0, 1.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		assert!(result.is_none()); // Should be completely clipped
	}

	#[test]
	fn test_is_inside_left_plane() {
		let inside = ClipVertex {
			world_pos: Vec3::ZERO,
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(0.5, 0.0, 0.0, 1.0),
			normal: Vec3::ZERO,
		};

		let outside = ClipVertex {
			world_pos: Vec3::ZERO,
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(-2.0, 0.0, 0.0, 1.0),
			normal: Vec3::ZERO,
		};

		assert!(is_inside(&inside, ClipPlane::Left));
		assert!(!is_inside(&outside, ClipPlane::Left));
	}

	#[test]
	fn test_signed_distance() {
		let clip_pos = Vec4::new(0.5, 0.0, 0.0, 1.0);

		// For left plane: x + w = 0.5 + 1.0 = 1.5 (inside, positive)
		let dist_left = signed_distance(clip_pos, ClipPlane::Left);
		assert!(dist_left > 0.0);

		// For right plane: w - x = 1.0 - 0.5 = 0.5 (inside, positive)
		let dist_right = signed_distance(clip_pos, ClipPlane::Right);
		assert!(dist_right > 0.0);
	}

	#[test]
	fn test_clip_partially_visible_face() {
		// Create a triangle that crosses the right frustum edge
		// One vertex inside, two vertices outside
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(0.0, 0.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(2.0, 0.0, 0.0), // Outside right edge
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(2.0, 2.0, 0.0), // Outside right edge and top
					normal: Vec3::ZERO,
					uv: (1.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		assert!(result.is_some());

		let clipped = result.unwrap();
		// Should have generated new vertices at intersection points
		assert!(clipped.len() >= 3);
	}

	#[test]
	fn test_uv_interpolation_accuracy() {
		// Test that UVs are correctly interpolated at intersection points
		let v1 = ClipVertex {
			world_pos: Vec3::new(0.0, 0.0, 0.0),
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(-0.5, 0.0, 0.0, 1.0), // Inside
			normal: Vec3::ZERO,
		};

		let v2 = ClipVertex {
			world_pos: Vec3::new(2.0, 0.0, 0.0),
			uv: (1.0, 1.0),
			clip_pos: Vec4::new(1.5, 0.0, 0.0, 1.0), // Outside right plane (x > w)
			normal: Vec3::ZERO,
		};

		// Compute intersection with right plane (x = w)
		let intersection = compute_intersection(&v1, &v2, ClipPlane::Right);
		assert!(intersection.is_some());

		let intersect = intersection.unwrap();
		// UV should be interpolated based on the t parameter
		// At intersection, clip_pos.x should equal clip_pos.w
		assert!((intersect.clip_pos.x - intersect.clip_pos.w).abs() < 1e-3);

		// UVs should be interpolated linearly
		assert!(intersect.uv.0 > 0.0 && intersect.uv.0 < 1.0);
		assert!(intersect.uv.1 > 0.0 && intersect.uv.1 < 1.0);
	}

	#[test]
	fn test_clip_against_all_six_planes() {
		// Test a large triangle that extends beyond all frustum bounds
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(-3.0, -3.0, -3.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(3.0, -3.0, -3.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(0.0, 3.0, 3.0),
					normal: Vec3::ZERO,
					uv: (0.5, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		// Should be clipped but not completely rejected
		// (depends on exact frustum configuration, but with identity matrix
		// some part should remain visible)
		assert!(result.is_some());

		let clipped = result.unwrap();
		assert!(clipped.len() >= 3);
	}

	#[test]
	fn test_sutherland_hodgman_preserves_winding() {
		// Test that the algorithm preserves vertex winding order
		let vertices = vec![
			ClipVertex {
				world_pos: Vec3::new(0.0, 0.0, 0.0),
				uv: (0.0, 0.0),
				clip_pos: Vec4::new(0.0, 0.0, 0.0, 1.0),
				normal: Vec3::ZERO,
			},
			ClipVertex {
				world_pos: Vec3::new(1.0, 0.0, 0.0),
				uv: (1.0, 0.0),
				clip_pos: Vec4::new(0.5, 0.0, 0.0, 1.0),
				normal: Vec3::ZERO,
			},
			ClipVertex {
				world_pos: Vec3::new(0.5, 1.0, 0.0),
				uv: (0.5, 1.0),
				clip_pos: Vec4::new(0.25, 0.5, 0.0, 1.0),
				normal: Vec3::ZERO,
			},
		];

		// Clip against a plane that doesn't intersect (all inside)
		let clipped = sutherland_hodgman_clip(vertices.clone(), ClipPlane::Left);

		// Should preserve all vertices in order
		assert_eq!(clipped.len(), 3);
	}

	#[test]
	fn test_degenerate_face_handling() {
		// Test that faces with < 3 vertices after clipping are properly rejected
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(-5.0, -5.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(-4.5, -5.0, 0.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(-4.7, -4.5, 0.0),
					normal: Vec3::ZERO,
					uv: (0.5, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		// Small triangle far outside should be completely clipped
		assert!(result.is_none());
	}

	#[test]
	fn test_trivial_rejection() {
		// Test early rejection optimization
		let vertices = vec![
			ClipVertex {
				world_pos: Vec3::ZERO,
				uv: (0.0, 0.0),
				clip_pos: Vec4::new(10.0, 0.0, 0.0, 1.0), // All outside right
				normal: Vec3::ZERO,
			},
			ClipVertex {
				world_pos: Vec3::ZERO,
				uv: (0.0, 0.0),
				clip_pos: Vec4::new(11.0, 0.0, 0.0, 1.0),
				normal: Vec3::ZERO,
			},
			ClipVertex {
				world_pos: Vec3::ZERO,
				uv: (0.0, 0.0),
				clip_pos: Vec4::new(10.5, 1.0, 0.0, 1.0),
				normal: Vec3::ZERO,
			},
		];

		assert!(is_trivially_rejected(&vertices));
	}

	#[test]
	fn test_nearly_parallel_edge_clipping() {
		// Test edge that's nearly parallel to a clipping plane
		let v1 = ClipVertex {
			world_pos: Vec3::new(0.0, 0.0, 0.0),
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(0.99999, 0.0, 0.0, 1.0), // Barely inside right plane
			normal: Vec3::ZERO,
		};

		let v2 = ClipVertex {
			world_pos: Vec3::new(0.1, 0.0, 0.0),
			uv: (1.0, 0.0),
			clip_pos: Vec4::new(1.00001, 0.0, 0.0, 1.0), // Barely outside right plane
			normal: Vec3::ZERO,
		};

		// Should successfully compute intersection, not return None
		let intersection = compute_intersection(&v1, &v2, ClipPlane::Right);
		assert!(
			intersection.is_some(),
			"Nearly parallel edge should be clipped, not dropped"
		);

		let intersect = intersection.unwrap();
		// Intersection should be very close to the plane boundary
		assert!((intersect.clip_pos.x - intersect.clip_pos.w).abs() < 1e-3);
	}

	#[test]
	fn test_edge_at_screen_boundary() {
		// Test case from the user's screenshot: edge at the left side of the screen
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(-1.5, 0.5, 0.0), // Outside left
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(-0.5, 0.5, 0.0), // Inside
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(-0.5, -0.5, 0.0), // Inside
					normal: Vec3::ZERO,
					uv: (1.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let vp_matrix = Mat4::IDENTITY;

		let result = clip_face_to_frustum(&face, &vp_matrix);
		assert!(
			result.is_some(),
			"Face crossing left boundary should be clipped"
		);

		let clipped = result.unwrap();
		// Should have 4 vertices after clipping (added 2 intersection points)
		assert!(
			clipped.len() >= 3,
			"Clipped face should have at least 3 vertices"
		);
	}

	#[test]
	fn test_no_silent_intersection_failures() {
		// Test that we never silently fail to compute intersections
		let test_cases = vec![
			// Nearly tangent cases
			(
				Vec4::new(1.0 - 1e-7, 0.0, 0.0, 1.0),
				Vec4::new(1.0 + 1e-7, 0.0, 0.0, 1.0),
			),
			(
				Vec4::new(-1.0 - 1e-7, 0.0, 0.0, 1.0),
				Vec4::new(-1.0 + 1e-7, 0.0, 0.0, 1.0),
			),
			// Edge cases with small denominators
			(
				Vec4::new(0.999999, 0.0, 0.0, 1.0),
				Vec4::new(1.000001, 0.0, 0.0, 1.0),
			),
		];

		for (clip1, clip2) in test_cases {
			let v1 = ClipVertex {
				world_pos: Vec3::ZERO,
				uv: (0.0, 0.0),
				clip_pos: clip1,
				normal: Vec3::ZERO,
			};
			let v2 = ClipVertex {
				world_pos: Vec3::ZERO,
				uv: (1.0, 1.0),
				clip_pos: clip2,
				normal: Vec3::ZERO,
			};

			// All these cases should compute intersections successfully
			let result = compute_intersection_t(&v1, &v2, ClipPlane::Right);
			assert!(result.is_some(), "Should never return None for valid edges");
		}
	}
	#[test]
	fn test_perspective_trivial_rejection() {
		// Test case simulating perspective projection where w varies
		// One vertex is close (low w), one is far (high w)
		// This simulates a face stretching away from the camera

		// Vertex 1: Close to camera, slightly left
		// x = -2.0, w = 1.0 -> x/w = -2.0 (Outside Left)
		let v1 = ClipVertex {
			world_pos: Vec3::ZERO,
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(-2.0, 0.0, 0.0, 1.0),
			normal: Vec3::ZERO,
		};

		// Vertex 2: Far from camera, centered
		// x = 0.0, w = 10.0 -> x/w = 0.0 (Inside)
		let v2 = ClipVertex {
			world_pos: Vec3::ZERO,
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(0.0, 0.0, 0.0, 10.0),
			normal: Vec3::ZERO,
		};

		// Vertex 3: Far from camera, slightly right
		// x = 2.0, w = 10.0 -> x/w = 0.2 (Inside)
		let v3 = ClipVertex {
			world_pos: Vec3::ZERO,
			uv: (0.0, 0.0),
			clip_pos: Vec4::new(2.0, 0.0, 0.0, 10.0),
			normal: Vec3::ZERO,
		};

		let vertices = vec![v1, v2, v3];

		// Should NOT be trivially rejected because v2 and v3 are inside
		assert!(
			!is_trivially_rejected(&vertices),
			"Face should not be trivially rejected if some vertices are inside"
		);
	}

	#[test]
	fn test_realistic_frustum_clipping() {
		// Setup camera similar to user's headshot settings
		let position = Vec3::new(0.0, 105.0, 120.0);
		let target = Vec3::new(0.0, 105.0, 0.0); // Looking at head
		let up = Vec3::Y;
		let fov_deg: f32 = 30.0;
		let aspect = 1.0; // Square render
		let near = 0.1;
		let far = 1000.0;

		let view = Mat4::look_at_rh(position, target, up);
		let proj = Mat4::perspective_rh(fov_deg.to_radians(), aspect, near, far);
		let vp_matrix = proj * view;

		// Create a face representing the arm (approximate position)
		// Arm is roughly at x=12, y=90..100, z=0
		let face = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(12.0, 90.0, 0.0),
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(16.0, 90.0, 0.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(16.0, 110.0, 0.0),
					normal: Vec3::ZERO,
					uv: (1.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		// Clip
		let result = clip_face_to_frustum(&face, &vp_matrix);

		// Should be fully visible (not clipped)
		assert!(result.is_some(), "Face should be visible");
		let clipped = result.unwrap();
		// The face might be clipped if it intersects the near plane (z=0 vs camera z=120, near=0.1)
		// Camera is at z=120, looking at z=0. View direction -Z.
		// Geometry at z=0 is 120 units in front. Safe.
		// So it should have 3 vertices.
		assert_eq!(clipped.len(), 3, "Face should not be cut");

		// Also test a point behind the camera
		let face_behind = geometry::Face {
			vertices: vec![
				geometry::Vertex {
					position: Vec3::new(0.0, 105.0, 200.0), // Behind camera (z=200 > camera z=120)
					normal: Vec3::ZERO,
					uv: (0.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(10.0, 105.0, 200.0),
					normal: Vec3::ZERO,
					uv: (1.0, 0.0),
				},
				geometry::Vertex {
					position: Vec3::new(0.0, 115.0, 200.0),
					normal: Vec3::ZERO,
					uv: (0.0, 1.0),
				},
			],
			texture_face: String::from("front"),
		};

		let result_behind = clip_face_to_frustum(&face_behind, &vp_matrix);
		assert!(
			result_behind.is_none(),
			"Points behind camera should be rejected"
		);
	}
}
