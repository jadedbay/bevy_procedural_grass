use bevy::render::{mesh::{Mesh, Indices}, render_resource::PrimitiveTopology};

const GRASS_MESH_POSITIONS: [[f32; 3]; 15] = [
    [0.034, 0.0, 0.0],
    [-0.034, 0.0, 0.0],
    [0.032, 0.14, 0.0],
    [-0.032, 0.14, 0.0],
    [0.029, 0.25, 0.0],
    [-0.029, 0.25, 0.0],
    [0.026, 0.34, 0.0],
    [-0.026, 0.34, 0.0],
    [0.023, 0.42, 0.0],
    [-0.023, 0.42, 0.0],
    [0.02, 0.48, 0.0],
    [-0.02, 0.48, 0.0],
    [0.013, 0.55, 0.0],
    [-0.013, 0.55, 0.0],
    [0.0, 0.63, 0.0],
];

const GRASS_MESH_UVS: [[f32; 2]; 15] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [0.0, 0.14 / 0.63],
    [1.0, 0.14 / 0.63],
    [0.0, 0.25 / 0.63],
    [1.0, 0.25 / 0.63],
    [0.0, 0.34 / 0.63],
    [1.0, 0.34 / 0.63],
    [0.0, 0.42 / 0.63],
    [1.0, 0.42 / 0.63],
    [0.0, 0.48 / 0.63],
    [1.0, 0.48 / 0.63],
    [0.0, 0.55 / 0.63],
    [1.0, 0.55 / 0.63],
    [0.5, 1.0],
];

const GRASS_MESH_INDICES: [u32; 39] = [
    2, 0, 1, 
    1, 3, 2, 
    4, 2, 3, 
    3, 5, 4, 
    6, 4, 5, 
    5, 7, 6, 
    8, 6, 7, 
    7, 9, 8, 
    10, 8, 9, 
    9, 11, 10, 
    12, 10, 11, 
    11, 13, 12, 
    14, 12, 13,
];

pub struct GrassMesh;

impl GrassMesh {
    pub fn mesh() -> Mesh {
        let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, GRASS_MESH_POSITIONS.to_vec());
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, GRASS_MESH_UVS.to_vec());
        grass_mesh.set_indices(Some(Indices::U32(GRASS_MESH_INDICES.to_vec())));
    
        grass_mesh
    }
}