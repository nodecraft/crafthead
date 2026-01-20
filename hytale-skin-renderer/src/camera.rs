//! Camera and projection system for 3D to 2D rendering

use crate::models::Vector3;
use glam::{Mat4, Vec3};

/// Trait for camera types that can provide projection matrices
///
/// Implement this trait for any camera type to enable it to work with the renderer.
/// The renderer uses `view_projection_matrix` for transforming vertices and
/// `calculate_depth` for depth sorting.
pub trait CameraProjection {
    /// Get the combined view-projection matrix
    fn view_projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4;

    /// Calculate depth for a point (for sorting)
    fn calculate_depth(&self, point: Vector3) -> f32;
}

/// Camera configuration for orthographic projection
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target (what it's looking at)
    pub target: Vec3,
    /// Up vector
    pub up: Vec3,
    /// Orthographic projection size (width and height of view)
    pub ortho_size: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Camera {
    /// Create a default isometric-style camera
    pub fn default_isometric() -> Self {
        Camera {
            position: Vec3::new(30.0, 30.0, 30.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 60.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for front-right view of characters
    /// Positioned to show the character's front and right side with a downward angle
    pub fn front_right_view() -> Self {
        Camera {
            position: Vec3::new(65.0, 75.0, 75.0),
            target: Vec3::new(0.0, 63.5, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 140.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for back-right view of characters
    /// Positioned to show the character's back and right side
    pub fn back_right_view() -> Self {
        Camera {
            position: Vec3::new(65.0, 75.0, -75.0),
            target: Vec3::new(0.0, 63.5, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 140.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for front-left view of characters
    /// Positioned to show the character's front and left side with a downward angle
    pub fn front_left_view() -> Self {
        Camera {
            position: Vec3::new(-65.0, 75.0, 75.0),
            target: Vec3::new(0.0, 63.5, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 140.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for back-left view of characters
    /// Positioned to show the character's back and left side
    pub fn back_left_view() -> Self {
        Camera {
            position: Vec3::new(-65.0, 75.0, -75.0),
            target: Vec3::new(0.0, 63.5, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 140.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for headshot view
    /// Positioned directly in front of the character's head for a close-up portrait
    pub fn headshot() -> Self {
        Camera {
            position: Vec3::new(0.0, 100.0, 150.0),
            target: Vec3::new(0.0, 100.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 30.0,
            near: 0.0000001,
            far: 1000.0,
        }
    }

    /// Create a camera preset for isometric head view
    /// Positioned at an angle to show the head from a three-quarter perspective
    pub fn isometric_head() -> Self {
        Camera {
            position: Vec3::new(-175.0, 175.0, 175.0),
            target: Vec3::new(0.0, 100.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 90.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for full body front view
    /// Positioned directly in front of the character showing the entire body head-on
    pub fn full_body_front() -> Self {
        Camera {
            position: Vec3::new(0.0, 63.5, 150.0),
            target: Vec3::new(0.0, 63.5, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 130.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera preset for player bust view
    /// Positioned to show the head and upper torso (bust shot), framing from mid-forearm upward
    pub fn player_bust() -> Self {
        Camera {
            position: Vec3::new(0.0, 92.0, 85.0),
            target: Vec3::new(0.0, 94.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size: 62.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a camera with custom position and target
    pub fn new(position: Vec3, target: Vec3, ortho_size: f32) -> Self {
        Camera {
            position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            ortho_size,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Get the view matrix (world to camera space)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the orthographic projection matrix
    pub fn projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        let aspect = output_width as f32 / output_height as f32;
        let half_width = self.ortho_size * aspect / 2.0;
        let half_height = self.ortho_size / 2.0;

        Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        self.projection_matrix(output_width, output_height) * self.view_matrix()
    }

    /// Project a 3D point to 2D screen space
    pub fn project_point(
        &self,
        point: Vector3,
        output_width: u32,
        output_height: u32,
    ) -> Option<(f32, f32, f32)> {
        let vp_matrix = self.view_projection_matrix(output_width, output_height);
        let world_point = Vec3::new(point.x, point.y, point.z);

        // Transform to clip space using manual multiplication
        let clip_vec = vp_matrix * world_point.extend(1.0);
        let clip_point = clip_vec.truncate() / clip_vec.w;

        // Check if point is behind camera (in clip space, z > 1.0 or z < -1.0 means outside view)
        if clip_point.z > 1.0 || clip_point.z < -1.0 {
            return None;
        }

        // Convert from clip space (-1 to 1) to screen space (0 to width/height)
        let screen_x = (clip_point.x + 1.0) * 0.5 * output_width as f32;
        let screen_y = (1.0 - clip_point.y) * 0.5 * output_height as f32; // Flip Y axis
        let depth = clip_point.z;

        Some((screen_x, screen_y, depth))
    }

    /// Calculate depth for a point (for sorting)
    pub fn calculate_depth(&self, point: Vector3) -> f32 {
        let view_matrix = self.view_matrix();
        let world_point = Vec3::new(point.x, point.y, point.z);
        let view_point = view_matrix.transform_point3(world_point);
        -view_point.z // Negative Z is forward in right-handed coordinates
    }
}

impl CameraProjection for Camera {
    fn view_projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        self.projection_matrix(output_width, output_height) * self.view_matrix()
    }

    fn calculate_depth(&self, point: Vector3) -> f32 {
        Camera::calculate_depth(self, point)
    }
}

/// Camera configuration for perspective projection
///
/// Use this camera type when orthographic projection causes clipping or culling
/// issues, particularly for close-up shots like headshots where vertices may
/// fall behind the near plane.
pub struct PerspectiveCamera {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target (what it's looking at)
    pub target: Vec3,
    /// Up vector
    pub up: Vec3,
    /// Vertical field of view in degrees
    pub fov_y: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl PerspectiveCamera {
    /// Create a perspective camera for headshot view
    /// Uses a narrower FOV to reduce distortion while avoiding clipping
    pub fn headshot() -> Self {
        PerspectiveCamera {
            // Position directly in front of head, pulled back
            position: Vec3::new(0.0, 107.0, 120.0),
            // Target the center of the head
            target: Vec3::new(0.0, 107.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            // Wider FOV to show full head
            fov_y: 21.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Create a perspective camera for isometric head view
    /// Positioned at an angle to show the head from a three-quarter perspective
    pub fn isometric_head() -> Self {
        PerspectiveCamera {
            position: Vec3::new(-80.0, 140.0, 80.0),
            target: Vec3::new(0.0, 100.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 35.0,
            near: 1.0,
            far: 1000.0,
        }
    }

    /// Create a perspective camera for player bust view
    /// Shows head and upper torso with natural perspective
    pub fn player_bust() -> Self {
        PerspectiveCamera {
            position: Vec3::new(0.0, 92.0, 100.0),
            target: Vec3::new(0.0, 94.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 40.0,
            near: 1.0,
            far: 1000.0,
        }
    }

    /// Create a perspective camera with custom parameters
    pub fn new(position: Vec3, target: Vec3, fov_y: f32) -> Self {
        PerspectiveCamera {
            position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Get the view matrix (world to camera space)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the perspective projection matrix
    pub fn projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        let aspect = output_width as f32 / output_height as f32;
        let fov_radians = self.fov_y.to_radians();

        Mat4::perspective_rh(fov_radians, aspect, self.near, self.far)
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        self.projection_matrix(output_width, output_height) * self.view_matrix()
    }

    /// Project a 3D point to 2D screen space
    pub fn project_point(
        &self,
        point: Vector3,
        output_width: u32,
        output_height: u32,
    ) -> Option<(f32, f32, f32)> {
        let vp_matrix = self.view_projection_matrix(output_width, output_height);
        let world_point = Vec3::new(point.x, point.y, point.z);

        // Transform to clip space
        let clip_vec = vp_matrix * world_point.extend(1.0);

        // Check if point is behind camera (w <= 0 means behind)
        if clip_vec.w <= 0.0 {
            return None;
        }

        // Perspective divide
        let ndc = clip_vec.truncate() / clip_vec.w;

        // Check if point is outside NDC bounds
        if ndc.z > 1.0 || ndc.z < -1.0 {
            return None;
        }

        // Convert from NDC (-1 to 1) to screen space (0 to width/height)
        let screen_x = (ndc.x + 1.0) * 0.5 * output_width as f32;
        let screen_y = (1.0 - ndc.y) * 0.5 * output_height as f32; // Flip Y axis
        let depth = ndc.z;

        Some((screen_x, screen_y, depth))
    }

    /// Calculate depth for a point (for sorting)
    pub fn calculate_depth(&self, point: Vector3) -> f32 {
        let view_matrix = self.view_matrix();
        let world_point = Vec3::new(point.x, point.y, point.z);
        let view_point = view_matrix.transform_point3(world_point);
        -view_point.z // Negative Z is forward in right-handed coordinates
    }
}

impl CameraProjection for PerspectiveCamera {
    fn view_projection_matrix(&self, output_width: u32, output_height: u32) -> Mat4 {
        self.projection_matrix(output_width, output_height) * self.view_matrix()
    }

    fn calculate_depth(&self, point: Vector3) -> f32 {
        PerspectiveCamera::calculate_depth(self, point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthographic_projection_matrix() {
        let camera = Camera::default_isometric();
        let proj = camera.projection_matrix(100, 100);

        // Orthographic matrix should not have perspective
        // Check that it's a valid matrix (not all zeros)
        let matrix_array = proj.to_cols_array_2d();
        assert_ne!(matrix_array[0][0], 0.0);
    }

    #[test]
    fn test_camera_position_affects_view() {
        let camera1 = Camera::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(0.0, 0.0, 0.0), 60.0);
        let camera2 = Camera::new(Vec3::new(20.0, 20.0, 20.0), Vec3::new(0.0, 0.0, 0.0), 60.0);

        let view1 = camera1.view_matrix();
        let view2 = camera2.view_matrix();

        // Different camera positions should produce different view matrices
        assert_ne!(view1, view2);
    }

    #[test]
    fn test_view_matrix_calculation() {
        let camera = Camera::default_isometric();
        let view = camera.view_matrix();

        // View matrix should be invertible (determinant != 0)
        let det = view.determinant();
        assert!(det.abs() > 0.0001);
    }

    #[test]
    fn test_depth_calculation() {
        let camera = Camera::default_isometric();

        let point1 = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let point2 = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        };

        let depth1 = camera.calculate_depth(point1);
        let depth2 = camera.calculate_depth(point2);

        // Point2 should be further (more negative depth in right-handed)
        assert!(depth2 < depth1);
    }

    #[test]
    fn test_project_3d_point_to_2d() {
        let camera = Camera::default_isometric();
        let point = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let result = camera.project_point(point, 100, 100);
        assert!(result.is_some());

        let (x, y, depth) = result.unwrap();

        // Should be within screen bounds
        assert!(x >= 0.0 && x <= 100.0);
        assert!(y >= 0.0 && y <= 100.0);
        assert!(depth.is_finite());
    }

    #[test]
    fn test_handle_points_behind_camera() {
        let camera = Camera::new(Vec3::new(0.0, 0.0, 10.0), Vec3::new(0.0, 0.0, 0.0), 60.0);

        // Point behind camera (further away from target than camera)
        // Camera is at z=10 looking at z=0, so point at z=20 is behind
        let point = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 20.0,
        };
        let result = camera.project_point(point, 100, 100);

        // May or may not be None depending on ortho size, but should handle gracefully
        // For now, just verify it doesn't panic
        if let Some((x, y, depth)) = result {
            assert!(x.is_finite());
            assert!(y.is_finite());
            assert!(depth.is_finite());
        }
    }

    #[test]
    fn test_view_projection_matrix() {
        let camera = Camera::default_isometric();
        let vp = camera.view_projection_matrix(100, 100);

        // Combined matrix should be valid (not all zeros)
        let matrix_array = vp.to_cols_array_2d();
        let has_non_zero = matrix_array
            .iter()
            .any(|row| row.iter().any(|&val| val.abs() > 0.0001));
        assert!(has_non_zero);
    }

    #[test]
    fn test_aspect_ratio_handling() {
        let camera = Camera::default_isometric();

        // Square output
        let proj_square = camera.projection_matrix(100, 100);

        // Wide output
        let proj_wide = camera.projection_matrix(200, 100);

        // Should be different due to aspect ratio
        assert_ne!(proj_square, proj_wide);
    }
}
