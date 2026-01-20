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
    let v0x = x2 - x0;
    let v0y = y2 - y0;
    let v1x = x1 - x0;
    let v1y = y1 - y0;
    let v2x = px - x0;
    let v2y = py - y0;

    let dot00 = v0x * v0x + v0y * v0y;
    let dot01 = v0x * v1x + v0y * v1y;
    let dot02 = v0x * v2x + v0y * v2y;
    let dot11 = v1x * v1x + v1y * v1y;
    let dot12 = v1x * v2x + v1y * v2y;

    // Calculate denominator (squared area of parallelogram formed by edge vectors)
    let denom = dot00 * dot11 - dot01 * dot01;

    // Check for degenerate triangle (collinear or coincident points)
    // Use a small epsilon to handle floating-point precision issues
    const EPSILON: f32 = 1e-10;
    if denom.abs() < EPSILON {
        // Degenerate triangle: return negative barycentric coordinates
        // to indicate point is outside (will be rejected by u >= 0 && v >= 0 && w >= 0 check)
        return (-1.0, -1.0, -1.0);
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    let w = 1.0 - u - v;

    (u, v, w)
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
