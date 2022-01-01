//! 2d and 3d entities to be added to a [super::common::Widget].

#![allow(dead_code)]

/// Color.
#[repr(C)]
pub struct Color {
    /// red [0..1]
    pub r: f32,
    /// green [0..1]
    pub g: f32,
    /// blue [0..1]
    pub b: f32,
    /// alpha [0..1]
    pub alpha: f32,
}

/// Colored vertices.
pub struct PositionColorVertices {
    /// Vector of vertices of position (3 elements) and color (4 elements)
    pub vertices: std::vec::Vec<[f32; 7]>,
}

impl PositionColorVertices {
    /// Constructs a single vertex (7-array) from a position and a color.
    fn to_array(position: nalgebra::Vector3<f32>, color: Color) -> [f32; 7] {
        [
            position.x,
            position.y,
            position.z,
            color.r,
            color.g,
            color.b,
            color.alpha,
        ]
    }
}

/// Position and texture coordinate vertices.
#[repr(C)]
pub struct PositionUvVertices {
    /// Vector of vertices of position (3 elements) and texture coordinate (2 elements)
    pub vertices: std::vec::Vec<[f32; 5]>,
}

impl PositionUvVertices {
    /// Constructs a single vertex (5-array) from a position and a texture coordinate.
    fn to_array(position: nalgebra::Vector3<f32>, uv: nalgebra::Vector2<f32>) -> [f32; 5] {
        [position.x, position.y, position.z, uv.x, uv.y]
    }
}

/// A texture.
pub struct Texture {}

/// Position/texture coordinate vertices and texture.
pub struct PositionUvVerticesAndTexture {
    /// The vertices.
    pub vertices: PositionUvVertices,
    /// The texture.
    pub texture: Texture,
}

/// Enumeration of possible vertex options.
#[derive(enum_as_inner::EnumAsInner)]
pub enum Vertices {
    /// Colored vertices.
    PositionColor(PositionColorVertices),
    /// Position/texture coordinate vertices and texture.
    PositionUvAndTexture(PositionUvVerticesAndTexture),
}

/// Triangle faces.
#[repr(C)]
pub struct Faces {
    /// Vector of triangle faces (3-array). A triangle face consists of three vertex indices.
    pub indices: std::vec::Vec<[i16; 3]>,
}

impl Faces {
    fn new(indices: std::vec::Vec<[i16; 3]>) -> Self {
        Self { indices }
    }
}

/// 3d entity to be added to a `Widget3`.
pub struct Entity3 {
    /// The vertices.
    pub vertices: Vertices,
    /// Faces.
    pub faces: Faces,
}

impl Entity3 {
    fn from_position_color_vertices_and_faces(
        vertices: PositionColorVertices,
        faces: Faces,
    ) -> Self {
        Self {
            vertices: Vertices::PositionColor(vertices),
            faces,
        }
    }

    fn from_position_uv_vertices_texture_and_faces(
        vertices: PositionUvVertices,
        texture: Texture,
        faces: Faces,
    ) -> Self {
        Self {
            vertices: Vertices::PositionUvAndTexture(PositionUvVerticesAndTexture {
                vertices,
                texture,
            }),
            faces,
        }
    }
}

/// A named entity has a pose, a name and - well - an [Entity3].
pub struct NamedEntity3 {
    /// The name.
    pub label: String,
    /// The 3d entity (mesh, line segments or point cloud).
    pub entity: Entity3,
    /// Pose of the entity in the scene.
    pub scene_pose_entity: nalgebra::Isometry3<f32>,
}

/// Creates a colored cube with a given scale.
pub fn colored_cube(scale: f32) -> Entity3 {
    #[rustfmt::skip]
    let vertices = PositionColorVertices{vertices: vec![
       [-scale, -scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [ scale, -scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [ scale,  scale, -scale,    1.0, 0.5, 0.5, 1.0],
       [-scale,  scale, -scale,    1.0, 0.5, 0.5, 1.0],

       [-scale, -scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [ scale, -scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [ scale,  scale,  scale,    0.5, 1.0, 0.5, 1.0],
       [-scale,  scale,  scale,    0.5, 1.0, 0.5, 1.0],
  
       [-scale, -scale, -scale,    0.5, 0.5, 1.0, 1.0],
       [-scale,  scale, -scale,    0.5, 0.5, 1.0, 1.0],
       [-scale,  scale,  scale,    0.5, 0.5, 1.0, 1.0],
       [-scale, -scale,  scale,    0.5, 0.5, 1.0, 1.0],
  
       [ scale, -scale, -scale,    1.0, 0.5, 0.0, 1.0],
       [ scale,  scale, -scale,    1.0, 0.5, 0.0, 1.0],
       [ scale,  scale,  scale,    1.0, 0.5, 0.0, 1.0],
       [ scale, -scale,  scale,    1.0, 0.5, 0.0, 1.0],
       
       [-scale, -scale, -scale,    0.0, 0.5, 1.0, 1.0],
       [-scale, -scale,  scale,    0.0, 0.5, 1.0, 1.0],
       [ scale, -scale,  scale,    0.0, 0.5, 1.0, 1.0],
       [ scale, -scale, -scale,    0.0, 0.5, 1.0, 1.0],
         
       [-scale,  scale, -scale,    1.0, 0.0, 0.5, 1.0],
       [-scale,  scale,  scale,    1.0, 0.0, 0.5, 1.0],
       [ scale,  scale,  scale,    1.0, 0.0, 0.5, 1.0],
       [ scale,  scale, -scale,    1.0, 0.0, 0.5, 1.0],
    ]};

    let faces = Faces::new(vec![
        [0, 1, 2],
        [0, 2, 3],
        [6, 5, 4],
        [7, 6, 4],
        [8, 9, 10],
        [8, 10, 11],
        [14, 13, 12],
        [15, 14, 12],
        [16, 17, 18],
        [16, 18, 19],
        [22, 21, 20],
        [23, 22, 20],
    ]);

    Entity3::from_position_color_vertices_and_faces(vertices, faces)
}

/// A colored triangle.
pub struct ColoredTriangle {
    /// A triangle face consists of three vertices.
    pub face: [[f32; 3]; 3],
    /// Color of the triangle face.
    pub color: Color,
}

impl ColoredTriangle {
    fn vec_of_arrays(vec_of_triangles: &[ColoredTriangle]) -> std::vec::Vec<[f32; 7]> {
        let mut result = std::vec::Vec::<[f32; 7]>::with_capacity(3 * vec_of_triangles.len());
        for triangle in vec_of_triangles {
            for vertex in triangle.face {
                result.push([
                    vertex[0],
                    vertex[1],
                    vertex[2],
                    triangle.color.r,
                    triangle.color.g,
                    triangle.color.b,
                    triangle.color.alpha,
                ])
            }
        }
        result
    }
}

/// Fills an [Entity3] with colored triangles.
pub fn colored_triangles(triangles: std::vec::Vec<ColoredTriangle>) -> Entity3 {
    let vertices = PositionColorVertices {
        vertices: ColoredTriangle::vec_of_arrays(&triangles),
    };
    let mut faces: Vec<[i16; 3]> = std::vec::Vec::new();

    let len: i16 = triangles.len().try_into().unwrap();
    for i in 0..len {
        faces.push([i * 3, i * 3 + 1, i * 3 + 2])
    }
    Entity3::from_position_color_vertices_and_faces(vertices, Faces::new(faces))
}
