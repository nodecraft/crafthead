//! Rendering math utilities
//!
//! Pure mathematical functions for rendering operations.

/// Calculate barycentric coordinates for a point relative to a triangle
///
/// Returns (u, v, w) where:
/// - u is the weight for vertex 2
/// - v is the weight for vertex 1
/// - w is the weight for vertex 0
/// - w = 1.0 - u - v
///
/// If all coordinates are >= 0, the point is inside the triangle.
/// Negative values indicate the point is outside the triangle.
///
/// For degenerate triangles (collinear or coincident points), returns negative
/// values to indicate the point is outside.
pub(crate) fn barycentric_coords(
	px: f32,
	py: f32,
	x0: f32,
	y0: f32,
	x1: f32,
	y1: f32,
	x2: f32,
	y2: f32,
) -> (f32, f32, f32) {
	match Barycentric::new(x0, y0, x1, y1, x2, y2) {
		Some(triangle) => triangle.coords(px, py),
		// Degenerate triangle: negative coordinates mark every point as outside
		None => (-1.0, -1.0, -1.0),
	}
}

/// The per-triangle half of the barycentric solve, so rasterizing only pays for the per-pixel half.
#[derive(Clone, Copy)]
pub(crate) struct Barycentric {
	x0: f32,
	y0: f32,
	v0x: f32,
	v0y: f32,
	v1x: f32,
	v1y: f32,
	dot00: f32,
	dot01: f32,
	dot11: f32,
	inv_denom: f32,
}

impl Barycentric {
	/// Returns None for degenerate triangles (collinear or coincident points), which cover no pixels.
	pub(crate) fn new(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> Option<Self> {
		let v0x = x2 - x0;
		let v0y = y2 - y0;
		let v1x = x1 - x0;
		let v1y = y1 - y0;

		let dot00 = v0x * v0x + v0y * v0y;
		let dot01 = v0x * v1x + v0y * v1y;
		let dot11 = v1x * v1x + v1y * v1y;

		// Squared area of the parallelogram formed by the edge vectors
		let denom = dot00 * dot11 - dot01 * dot01;

		const EPSILON: f32 = 1e-10;
		if denom.abs() < EPSILON {
			return None;
		}

		Some(Barycentric {
			x0,
			y0,
			v0x,
			v0y,
			v1x,
			v1y,
			dot00,
			dot01,
			dot11,
			inv_denom: 1.0 / denom,
		})
	}

	/// Same (u, v, w) convention as `barycentric_coords`.
	#[inline]
	pub(crate) fn coords(&self, px: f32, py: f32) -> (f32, f32, f32) {
		let v2x = px - self.x0;
		let v2y = py - self.y0;

		let dot02 = self.v0x * v2x + self.v0y * v2y;
		let dot12 = self.v1x * v2x + self.v1y * v2y;

		let u = (self.dot11 * dot02 - self.dot01 * dot12) * self.inv_denom;
		let v = (self.dot00 * dot12 - self.dot01 * dot02) * self.inv_denom;
		let w = 1.0 - u - v;

		(u, v, w)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_barycentric_coords() {
		// Test barycentric coordinate calculation
		let (u, v, w) = barycentric_coords(5.0, 5.0, 0.0, 0.0, 10.0, 0.0, 5.0, 10.0);

		// Should sum to 1
		assert!((u + v + w - 1.0).abs() < 0.001);

		// All should be positive for point inside triangle
		assert!(u >= 0.0);
		assert!(v >= 0.0);
		assert!(w >= 0.0);
	}

	#[test]
	fn test_barycentric_coords_degenerate_triangle() {
		// Test with collinear points (degenerate triangle)
		let (u, v, w) = barycentric_coords(5.0, 5.0, 0.0, 0.0, 5.0, 5.0, 10.0, 10.0);

		// Should return negative values to indicate point is outside
		// (degenerate triangles have zero area, so all points are "outside")
		assert!(u < 0.0 || v < 0.0 || w < 0.0);

		// Test with coincident points
		let (u, v, w) = barycentric_coords(5.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
		assert!(u < 0.0 || v < 0.0 || w < 0.0);

		// Test with two coincident points
		let (u, v, w) = barycentric_coords(5.0, 5.0, 0.0, 0.0, 0.0, 0.0, 10.0, 10.0);
		assert!(u < 0.0 || v < 0.0 || w < 0.0);
	}

	#[test]
	fn test_barycentric_coords_no_division_by_zero() {
		// Test that degenerate triangles don't cause NaN or infinity
		let (u, v, w) = barycentric_coords(1.0, 1.0, 0.0, 0.0, 2.0, 2.0, 4.0, 4.0);

		// Should return finite values (not NaN or infinity)
		assert!(u.is_finite());
		assert!(v.is_finite());
		assert!(w.is_finite());

		// Should be negative to indicate outside
		assert!(u < 0.0 || v < 0.0 || w < 0.0);
	}
}
