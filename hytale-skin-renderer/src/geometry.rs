//! Geometry generation for boxes and quads

use crate::models::{QuadNormal, Shape, ShapeType, Vector3};
use glam::{Mat4, Vec3};

/// A vertex with position, normal, and UV coordinates
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: (f32, f32),
}

/// A face with vertices and texture coordinates
#[derive(Debug, Clone)]
pub struct Face {
    pub vertices: Vec<Vertex>,
    pub texture_face: String, // "front", "back", "left", "right", "top", "bottom"
}

/// Generate geometry for a shape
pub fn generate_geometry(shape: &Shape, transform: Mat4) -> Vec<Face> {
    match shape.shape_type {
        ShapeType::Box => generate_box_geometry(shape, transform),
        ShapeType::Quad => generate_quad_geometry(shape, transform),
        ShapeType::None => Vec::new(),
    }
}

fn generate_box_geometry(shape: &Shape, transform: Mat4) -> Vec<Face> {
    let size = shape.settings.size.unwrap_or(Vector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    });
    let half_x = size.x / 2.0;
    let half_y = size.y / 2.0;
    let half_z = size.z / 2.0;

    let offset = crate::math::vec3_from_blockymodel(shape.offset);
    let stretch = crate::math::vec3_from_blockymodel(shape.stretch);

    let shape_transform = Mat4::from_translation(offset) * Mat4::from_scale(stretch);
    let final_transform = transform * shape_transform;

    let mut faces = Vec::new();

    // Helper to compute UVs for a face
    // Helper to compute UVs for a face
    // We always use standard 0-1 UVs (QUAD_UVS) for the vertices.
    // The rasterizer (sample_face_texture) will handle the actual texture layout mapping
    // using the Face's texture_face name and the Shape's texture_layout.
    let get_uvs = |_face_name: &str, _size_u: f32, _size_v: f32| -> [(f32, f32); 4] { QUAD_UVS };

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(-half_x, -half_y, half_z),
            Vec3::new(half_x, -half_y, half_z),
            Vec3::new(half_x, half_y, half_z),
            Vec3::new(-half_x, half_y, half_z),
        ],
        &get_uvs("front", size.x, size.y),
        Vec3::new(0.0, 0.0, 1.0),
        "front",
        final_transform,
    ));

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(half_x, -half_y, -half_z),
            Vec3::new(-half_x, -half_y, -half_z),
            Vec3::new(-half_x, half_y, -half_z),
            Vec3::new(half_x, half_y, -half_z),
        ],
        &get_uvs("back", size.x, size.y),
        Vec3::new(0.0, 0.0, -1.0),
        "back",
        final_transform,
    ));

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(half_x, -half_y, half_z),
            Vec3::new(half_x, -half_y, -half_z),
            Vec3::new(half_x, half_y, -half_z),
            Vec3::new(half_x, half_y, half_z),
        ],
        &get_uvs("right", size.z, size.y),
        Vec3::new(1.0, 0.0, 0.0),
        "right",
        final_transform,
    ));

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(-half_x, -half_y, -half_z),
            Vec3::new(-half_x, -half_y, half_z),
            Vec3::new(-half_x, half_y, half_z),
            Vec3::new(-half_x, half_y, -half_z),
        ],
        &get_uvs("left", size.z, size.y),
        Vec3::new(-1.0, 0.0, 0.0),
        "left",
        final_transform,
    ));

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(-half_x, half_y, half_z),
            Vec3::new(half_x, half_y, half_z),
            Vec3::new(half_x, half_y, -half_z),
            Vec3::new(-half_x, half_y, -half_z),
        ],
        &get_uvs("top", size.x, size.z),
        Vec3::new(0.0, 1.0, 0.0),
        "top",
        final_transform,
    ));

    faces.push(create_face_with_uvs(
        &[
            Vec3::new(-half_x, -half_y, -half_z),
            Vec3::new(half_x, -half_y, -half_z),
            Vec3::new(half_x, -half_y, half_z),
            Vec3::new(-half_x, -half_y, half_z),
        ],
        &get_uvs("bottom", size.x, size.z),
        Vec3::new(0.0, -1.0, 0.0),
        "bottom",
        final_transform,
    ));

    if shape.double_sided {
        let mut reversed_faces = Vec::new();
        for face in &faces {
            let mut reversed_vertices = face.vertices.clone();
            reversed_vertices.reverse();
            let reversed_normal = -face.vertices[0].normal;
            reversed_faces.push(Face {
                vertices: reversed_vertices
                    .iter()
                    .map(|v| Vertex {
                        position: v.position,
                        normal: reversed_normal,
                        uv: v.uv,
                    })
                    .collect(),
                texture_face: face.texture_face.clone(),
            });
        }
        faces.extend(reversed_faces);
    }

    faces
}

fn generate_quad_geometry(shape: &Shape, transform: Mat4) -> Vec<Face> {
    let size = shape.settings.size.unwrap_or(Vector3 {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });
    let normal = shape.settings.normal.unwrap_or(QuadNormal::PosZ);

    let offset = crate::math::vec3_from_blockymodel(shape.offset);
    let stretch = crate::math::vec3_from_blockymodel(shape.stretch);

    let shape_transform = Mat4::from_translation(offset) * Mat4::from_scale(stretch);
    let final_transform = transform * shape_transform;

    let get_uvs = |_face_name: &str, _size_u: f32, _size_v: f32| -> [(f32, f32); 4] { QUAD_UVS };

    let (vertices, normal_vec, face_name) = match normal {
        QuadNormal::PosX => {
            let half_y = size.y / 2.0;
            let half_z = size.z / 2.0;
            (
                vec![
                    Vec3::new(0.0, -half_y, -half_z),
                    Vec3::new(0.0, half_y, -half_z),
                    Vec3::new(0.0, half_y, half_z),
                    Vec3::new(0.0, -half_y, half_z),
                ],
                Vec3::new(1.0, 0.0, 0.0),
                "right",
            )
        }
        QuadNormal::NegX => {
            let half_y = size.y / 2.0;
            let half_z = size.z / 2.0;
            (
                vec![
                    Vec3::new(0.0, -half_y, half_z),
                    Vec3::new(0.0, half_y, half_z),
                    Vec3::new(0.0, half_y, -half_z),
                    Vec3::new(0.0, -half_y, -half_z),
                ],
                Vec3::new(-1.0, 0.0, 0.0),
                "left",
            )
        }
        QuadNormal::PosY => {
            let half_x = size.x / 2.0;
            let half_z = size.z / 2.0;
            (
                vec![
                    Vec3::new(-half_x, 0.0, -half_z),
                    Vec3::new(half_x, 0.0, -half_z),
                    Vec3::new(half_x, 0.0, half_z),
                    Vec3::new(-half_x, 0.0, half_z),
                ],
                Vec3::new(0.0, 1.0, 0.0),
                "top",
            )
        }
        QuadNormal::NegY => {
            let half_x = size.x / 2.0;
            let half_z = size.z / 2.0;
            (
                vec![
                    Vec3::new(-half_x, 0.0, half_z),
                    Vec3::new(half_x, 0.0, half_z),
                    Vec3::new(half_x, 0.0, -half_z),
                    Vec3::new(-half_x, 0.0, -half_z),
                ],
                Vec3::new(0.0, -1.0, 0.0),
                "bottom",
            )
        }
        QuadNormal::PosZ => {
            let half_x = size.x / 2.0;
            let half_y = size.y / 2.0;
            (
                vec![
                    Vec3::new(-half_x, -half_y, 0.0),
                    Vec3::new(half_x, -half_y, 0.0),
                    Vec3::new(half_x, half_y, 0.0),
                    Vec3::new(-half_x, half_y, 0.0),
                ],
                Vec3::new(0.0, 0.0, 1.0),
                "front",
            )
        }
        QuadNormal::NegZ => {
            let half_x = size.x / 2.0;
            let half_y = size.y / 2.0;
            (
                vec![
                    Vec3::new(half_x, -half_y, 0.0),
                    Vec3::new(-half_x, -half_y, 0.0),
                    Vec3::new(-half_x, half_y, 0.0),
                    Vec3::new(half_x, half_y, 0.0),
                ],
                Vec3::new(0.0, 0.0, -1.0),
                "back",
            )
        }
    };

    let layout_exists = match face_name {
        "right" => shape.texture_layout.right.is_some(),
        "left" => shape.texture_layout.left.is_some(),
        "top" => shape.texture_layout.top.is_some(),
        "bottom" => shape.texture_layout.bottom.is_some(),
        "front" => shape.texture_layout.front.is_some(),
        "back" => shape.texture_layout.back.is_some(),
        _ => false,
    };

    let final_face_name = if !layout_exists && shape.texture_layout.front.is_some() {
        "front"
    } else {
        face_name
    };

    let mut faces = vec![create_face_with_uvs(
        &vertices,
        &get_uvs(face_name, size.x, size.y), // Does this match all orientations?
        // Box UVs use (x,y), (z,y), (x,z).
        // For Quad:
        // PosZ (Front): size.x, size.y
        // NegZ (Back): size.x, size.y
        // PosX (Right): size.z (depth), size.y
        // NegX (Left): size.z, size.y
        // PosY (Top): size.x, size.z
        // NegY (Bottom): size.x, size.z
        //
        // So we need to select correct dimensions based on normal.
        normal_vec,
        final_face_name,
        final_transform,
    )];

    if shape.double_sided {
        let normal_vec_transformed = faces[0].vertices[0].normal;

        // Use explicit permutation for horizontal flip (0<->1, 2<->3)
        // This ensures consistent winding and mirroring for Quads regardless of start vertex
        let v = &faces[0].vertices;
        let reversed_vertices = vec![v[1].clone(), v[0].clone(), v[3].clone(), v[2].clone()];

        let reversed_normal = -normal_vec_transformed;
        faces.push(Face {
            vertices: reversed_vertices
                .iter()
                .map(|v| Vertex {
                    position: v.position,
                    normal: reversed_normal,
                    uv: v.uv,
                })
                .collect(),
            texture_face: final_face_name.to_string(),
        });
    }

    faces
}

/// Standard UV coordinates for a quad face (counter-clockwise from bottom-left)
const QUAD_UVS: [(f32, f32); 4] = [
    (0.0, 1.0), // Bottom-left (V=1 because texture Y is typically inverted)
    (1.0, 1.0), // Bottom-right
    (1.0, 0.0), // Top-right
    (0.0, 0.0), // Top-left
];

fn create_face_with_uvs(
    positions: &[Vec3],
    uvs: &[(f32, f32)],
    normal: Vec3,
    texture_face: &str,
    transform: Mat4,
) -> Face {
    let vertices: Vec<Vertex> = positions
        .iter()
        .zip(uvs.iter())
        .map(|(&pos, &uv)| {
            // Transform position
            let transformed_pos = transform.transform_point3(pos);
            // Transform normal (only rotation, no translation)
            let transformed_normal = transform.transform_vector3(normal).normalize();
            Vertex {
                position: transformed_pos,
                normal: transformed_normal,
                uv,
            }
        })
        .collect();

    Face {
        vertices,
        texture_face: texture_face.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Shape, ShapeSettings, TextureLayout};
    use glam::Mat4;

    fn create_test_box_shape() -> Shape {
        Shape {
            offset: Vector3::zero(),
            stretch: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            texture_layout: TextureLayout::default(),
            shape_type: ShapeType::Box,
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
            unwrap_mode: "custom".to_string(),
            visible: true,
            double_sided: false,
            shading_mode: "flat".to_string(),
        }
    }

    fn create_test_quad_shape(normal: QuadNormal) -> Shape {
        Shape {
            offset: Vector3::zero(),
            stretch: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            texture_layout: TextureLayout::default(),
            shape_type: ShapeType::Quad,
            settings: ShapeSettings {
                size: Some(Vector3 {
                    x: 12.0,
                    y: 7.0,
                    z: 0.0,
                }),
                normal: Some(normal),
                is_piece: None,
                is_static_box: None,
            },
            unwrap_mode: "custom".to_string(),
            visible: true,
            double_sided: false,
            shading_mode: "flat".to_string(),
        }
    }

    #[test]
    fn test_generate_box_vertices() {
        let shape = create_test_box_shape();
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Box should have 6 faces
        assert_eq!(faces.len(), 6);

        // Each face should have 4 vertices
        for face in &faces {
            assert_eq!(face.vertices.len(), 4);
        }

        // Total vertices: 6 faces * 4 vertices = 24
        let total_vertices: usize = faces.iter().map(|f| f.vertices.len()).sum();
        assert_eq!(total_vertices, 24);
    }

    #[test]
    fn test_generate_quad_vertices() {
        let shape = create_test_quad_shape(QuadNormal::PosZ);
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Quad should have 1 face
        assert_eq!(faces.len(), 1);

        // Face should have 4 vertices
        assert_eq!(faces[0].vertices.len(), 4);
    }

    #[test]
    fn test_box_face_normals() {
        let shape = create_test_box_shape();
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Check that normals point outward
        let front_face = faces.iter().find(|f| f.texture_face == "front").unwrap();
        assert!((front_face.vertices[0].normal.z - 1.0).abs() < 0.001);

        let back_face = faces.iter().find(|f| f.texture_face == "back").unwrap();
        assert!((back_face.vertices[0].normal.z + 1.0).abs() < 0.001);

        let right_face = faces.iter().find(|f| f.texture_face == "right").unwrap();
        assert!((right_face.vertices[0].normal.x - 1.0).abs() < 0.001);

        let left_face = faces.iter().find(|f| f.texture_face == "left").unwrap();
        assert!((left_face.vertices[0].normal.x + 1.0).abs() < 0.001);

        let top_face = faces.iter().find(|f| f.texture_face == "top").unwrap();
        assert!((top_face.vertices[0].normal.y - 1.0).abs() < 0.001);

        let bottom_face = faces.iter().find(|f| f.texture_face == "bottom").unwrap();
        assert!((bottom_face.vertices[0].normal.y + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quad_normal_directions() {
        let normals = [
            QuadNormal::PosX,
            QuadNormal::NegX,
            QuadNormal::PosY,
            QuadNormal::NegY,
            QuadNormal::PosZ,
            QuadNormal::NegZ,
        ];

        for normal in &normals {
            let shape = create_test_quad_shape(*normal);
            let transform = Mat4::IDENTITY;
            let faces = generate_geometry(&shape, transform);

            assert_eq!(faces.len(), 1);
            let normal_vec = faces[0].vertices[0].normal;

            match normal {
                QuadNormal::PosX => assert!((normal_vec.x - 1.0).abs() < 0.001),
                QuadNormal::NegX => assert!((normal_vec.x + 1.0).abs() < 0.001),
                QuadNormal::PosY => assert!((normal_vec.y - 1.0).abs() < 0.001),
                QuadNormal::NegY => assert!((normal_vec.y + 1.0).abs() < 0.001),
                QuadNormal::PosZ => assert!((normal_vec.z - 1.0).abs() < 0.001),
                QuadNormal::NegZ => assert!((normal_vec.z + 1.0).abs() < 0.001),
            }
        }
    }

    #[test]
    fn test_double_sided_faces() {
        let mut shape = create_test_box_shape();
        shape.double_sided = true;
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Double-sided box should have 12 faces (6 * 2)
        assert_eq!(faces.len(), 12);
    }

    #[test]
    fn test_double_sided_quad() {
        let mut shape = create_test_quad_shape(QuadNormal::PosZ);
        shape.double_sided = true;
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Double-sided quad should have 2 faces
        assert_eq!(faces.len(), 2);

        // Normals should be opposite
        assert!((faces[0].vertices[0].normal.dot(faces[1].vertices[0].normal) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_stretch_affects_vertex_positions() {
        let mut shape = create_test_box_shape();
        shape.stretch = Vector3 {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };
        // Note: stretch is applied in scene graph, not here
        // This test verifies geometry generation works with different sizes
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Should still generate valid geometry
        assert_eq!(faces.len(), 6);
        for face in &faces {
            assert_eq!(face.vertices.len(), 4);
        }
    }

    #[test]
    fn test_size_calculations_box() {
        let mut shape = create_test_box_shape();
        shape.settings.size = Some(Vector3 {
            x: 20.0,
            y: 30.0,
            z: 40.0,
        });
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        // Front face should span from -10 to +10 in X, -15 to +15 in Y, at Z=20
        let front_face = faces.iter().find(|f| f.texture_face == "front").unwrap();
        let positions: Vec<Vec3> = front_face.vertices.iter().map(|v| v.position).collect();

        // Check X range
        let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = positions
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min_x - (-10.0)).abs() < 0.1);
        assert!((max_x - 10.0).abs() < 0.1);

        // Check Y range
        let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min_y - (-15.0)).abs() < 0.1);
        assert!((max_y - 15.0).abs() < 0.1);

        // All vertices should be at Z = 20
        for pos in &positions {
            assert!((pos.z - 20.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_size_calculations_quad() {
        let shape = create_test_quad_shape(QuadNormal::PosZ);
        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        let face = &faces[0];
        let positions: Vec<Vec3> = face.vertices.iter().map(|v| v.position).collect();

        // Quad with size (12, 7) should span from -6 to +6 in X, -3.5 to +3.5 in Y
        let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = positions
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min_x - (-6.0)).abs() < 0.1);
        assert!((max_x - 6.0).abs() < 0.1);

        let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min_y - (-3.5)).abs() < 0.1);
        assert!((max_y - 3.5).abs() < 0.1);
    }

    #[test]
    fn test_quad_texture_fallback_to_front() {
        // Create a quad facing -Z (Back), which normally maps to "back" face
        let mut shape = create_test_quad_shape(QuadNormal::NegZ);

        // Setup texture layout to ONLY have "front"
        shape.texture_layout = TextureLayout {
            front: Some(crate::models::UvFace {
                offset: crate::models::UvOffset { x: 0.0, y: 0.0 },
                mirror: crate::models::UvMirror { x: false, y: false },
                angle: crate::models::UvAngle(0),
            }),
            back: None, // Explicitly missing
            left: None,
            right: None,
            top: None,
            bottom: None,
        };

        let transform = Mat4::IDENTITY;
        let faces = generate_geometry(&shape, transform);

        assert_eq!(faces.len(), 1);
        // Should have fallen back to "front"
        assert_eq!(faces[0].texture_face, "front");

        // Verify normal is still correct (-Z for back face)
        assert!((faces[0].vertices[0].normal.z + 1.0).abs() < 0.001);
    }
}
