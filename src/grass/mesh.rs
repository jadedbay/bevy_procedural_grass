use bevy::render::{mesh::{Mesh, Indices}, render_resource::PrimitiveTopology};

pub struct GrassMesh;

impl GrassMesh {
    pub fn mesh(segments: u32) -> Mesh {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for i in 0..segments {
            let y = (i as f32 / segments as f32).sqrt();
            positions.push([1.0, y, 0.0]);
            positions.push([-1.0, y, 0.0]);
            uvs.push([0.0, y]);
            uvs.push([1.0, y]);

            if i > 0 {
                indices.push(2 * i);
                indices.push(2 * i - 1);
                indices.push(2 * i - 2);

                indices.push(2 * i + 1);
                indices.push(2 * i - 1);
                indices.push(2 * i);
            }
        }

        positions.push([0.0, 1.0, 0.0]);
        uvs.push([0.5, 1.0]);

        let tip = 2 * segments - 1;
        indices.push(tip);
        indices.push(tip - 1);
        indices.push(tip + 1);

        let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        grass_mesh.set_indices(Some(Indices::U32(indices)));

        grass_mesh
    }
}