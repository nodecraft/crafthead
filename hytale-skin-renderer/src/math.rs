//! Math utilities for transformations, quaternions, and matrices

use crate::models::{Quaternion, Vector3};
use glam::{Mat4, Quat, Vec3};

/// Convert a blockymodel Quaternion to a glam Quat
pub fn quat_from_blockymodel(q: Quaternion) -> Quat {
    Quat::from_xyzw(q.x, q.y, q.z, q.w).normalize()
}

/// Convert a blockymodel Vector3 to a glam Vec3
pub fn vec3_from_blockymodel(v: Vector3) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

/// Convert a glam Vec3 to a blockymodel Vector3
pub fn vec3_to_blockymodel(v: Vec3) -> Vector3 {
    Vector3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

/// Build a transformation matrix from position, rotation (quaternion), and scale
pub fn build_transform_matrix(position: Vector3, rotation: Quaternion, scale: Vector3) -> Mat4 {
    let pos = vec3_from_blockymodel(position);
    let rot = quat_from_blockymodel(rotation);
    let scale_vec = vec3_from_blockymodel(scale);

    Mat4::from_scale_rotation_translation(scale_vec, rot, pos)
}

/// Build a transformation matrix from position, rotation, scale, and offset
pub fn build_transform_with_offset(
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
    offset: Vector3,
) -> Mat4 {
    let transform = build_transform_matrix(position, rotation, scale);
    let offset_vec = vec3_from_blockymodel(offset);
    transform * Mat4::from_translation(offset_vec)
}

/// Multiply two transformation matrices
pub fn multiply_transforms(a: Mat4, b: Mat4) -> Mat4 {
    a * b
}

/// Transform a 3D point by a transformation matrix
pub fn transform_point(matrix: Mat4, point: Vector3) -> Vector3 {
    let vec = vec3_from_blockymodel(point);
    let transformed = matrix.transform_point3(vec);
    vec3_to_blockymodel(transformed)
}

/// Transform a direction vector by a transformation matrix (no translation)
pub fn transform_direction(matrix: Mat4, direction: Vector3) -> Vector3 {
    let vec = vec3_from_blockymodel(direction);
    let transformed = matrix.transform_vector3(vec);
    vec3_to_blockymodel(transformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Quaternion;

    #[test]
    fn test_quat_from_blockymodel_identity() {
        let q = Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        };
        let quat = quat_from_blockymodel(q);
        assert!((quat.x - 0.0).abs() < 0.0001);
        assert!((quat.y - 0.0).abs() < 0.0001);
        assert!((quat.z - 0.0).abs() < 0.0001);
        assert!((quat.w - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_quat_from_blockymodel_normalization() {
        // Test that quaternion is normalized
        let q = Quaternion {
            x: 2.0,
            y: 0.0,
            z: 0.0,
            w: 2.0,
        };
        let quat = quat_from_blockymodel(q);
        let length = (quat.x * quat.x + quat.y * quat.y + quat.z * quat.z + quat.w * quat.w).sqrt();
        assert!((length - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_quaternion_90_degree_rotation_x() {
        // 90 degree rotation around X axis
        let q = Quaternion {
            x: 0.7071068,
            y: 0.0,
            z: 0.0,
            w: 0.7071068,
        };
        let quat = quat_from_blockymodel(q);
        let matrix = Mat4::from_quat(quat);

        // Rotate a vector pointing up (0, 1, 0) by 90 degrees around X
        // Should result in (0, 0, 1) - pointing forward
        let up = Vec3::new(0.0, 1.0, 0.0);
        let rotated = matrix.transform_vector3(up);
        assert!((rotated.x - 0.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
        assert!((rotated.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_90_degree_rotation_y() {
        // 90 degree rotation around Y axis
        let q = Quaternion {
            x: 0.0,
            y: 0.7071068,
            z: 0.0,
            w: 0.7071068,
        };
        let quat = quat_from_blockymodel(q);
        let matrix = Mat4::from_quat(quat);

        // Rotate a vector pointing forward (0, 0, 1) by 90 degrees around Y
        // Should result in (1, 0, 0) - pointing right
        let forward = Vec3::new(0.0, 0.0, 1.0);
        let rotated = matrix.transform_vector3(forward);
        assert!((rotated.x - 1.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
        assert!((rotated.z - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_90_degree_rotation_z() {
        // 90 degree rotation around Z axis
        let q = Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.7071068,
            w: 0.7071068,
        };
        let quat = quat_from_blockymodel(q);
        let matrix = Mat4::from_quat(quat);

        // Rotate a vector pointing right (1, 0, 0) by 90 degrees around Z
        // Should result in (0, 1, 0) - pointing up
        let right = Vec3::new(1.0, 0.0, 0.0);
        let rotated = matrix.transform_vector3(right);
        assert!((rotated.x - 0.0).abs() < 0.001);
        assert!((rotated.y - 1.0).abs() < 0.001);
        assert!((rotated.z - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_multiplication() {
        // Test quaternion multiplication (composition of rotations)
        let q1 = Quat::from_rotation_z(std::f32::consts::PI / 2.0); // 90 deg Z
        let q2 = Quat::from_rotation_x(std::f32::consts::PI / 2.0); // 90 deg X
        let combined = q1 * q2;

        // Combined rotation should be valid quaternion
        assert!((combined.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_multiplication() {
        let identity = Mat4::IDENTITY;
        let translation = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));

        let result = multiply_transforms(identity, translation);
        assert_eq!(result, translation);

        let result2 = multiply_transforms(translation, identity);
        assert_eq!(result2, translation);
    }

    #[test]
    fn test_vector_transformations() {
        let v = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let vec = vec3_from_blockymodel(v);
        let back = vec3_to_blockymodel(vec);

        assert_eq!(v.x, back.x);
        assert_eq!(v.y, back.y);
        assert_eq!(v.z, back.z);
    }

    #[test]
    fn test_build_transform_matrix() {
        let position = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let rotation = Quaternion::identity();
        let scale = Vector3 {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };

        let matrix = build_transform_matrix(position, rotation, scale);

        // Transform origin point
        let origin = Vector3::zero();
        let transformed = transform_point(matrix, origin);

        // Should be at position (scaling doesn't affect translation of origin)
        assert!((transformed.x - 1.0).abs() < 0.0001);
        assert!((transformed.y - 2.0).abs() < 0.0001);
        assert!((transformed.z - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_point() {
        let translation = Mat4::from_translation(Vec3::new(5.0, 10.0, 15.0));
        let point = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let transformed = transform_point(translation, point);

        assert!((transformed.x - 6.0).abs() < 0.0001);
        assert!((transformed.y - 12.0).abs() < 0.0001);
        assert!((transformed.z - 18.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform_direction() {
        let scale = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let direction = Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };

        let transformed = transform_direction(scale, direction);

        // Direction should be scaled but not translated
        assert!((transformed.x - 2.0).abs() < 0.0001);
        assert!((transformed.y - 0.0).abs() < 0.0001);
        assert!((transformed.z - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_build_transform_with_offset() {
        let position = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let rotation = Quaternion::identity();
        let scale = Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let offset = Vector3 {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        };

        let matrix = build_transform_with_offset(position, rotation, scale, offset);
        let origin = Vector3::zero();
        let transformed = transform_point(matrix, origin);

        // Should be at offset position
        assert!((transformed.x - 10.0).abs() < 0.0001);
        assert!((transformed.y - 20.0).abs() < 0.0001);
        assert!((transformed.z - 30.0).abs() < 0.0001);
    }

    #[test]
    fn test_edge_case_zero_quaternion() {
        // Zero quaternion - glam normalizes it to identity (0, 0, 0, 1)
        let q = Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        let quat = quat_from_blockymodel(q);
        // Normalization of zero quaternion results in identity or NaN
        // Check that it's either identity or has NaN (which is acceptable for edge case)
        let is_identity = (quat.w - 1.0).abs() < 0.0001
            && quat.x.abs() < 0.0001
            && quat.y.abs() < 0.0001
            && quat.z.abs() < 0.0001;
        let has_nan = quat.x.is_nan() || quat.y.is_nan() || quat.z.is_nan() || quat.w.is_nan();
        // Either identity or NaN is acceptable for this edge case
        assert!(
            is_identity || has_nan,
            "Zero quaternion should normalize to identity or NaN"
        );
    }
}
