use bevy::render::{mesh::{Indices, Mesh}, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology};

pub struct GrassMesh;

impl GrassMesh {
    pub fn mesh(segments: u32) -> Mesh {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for i in 0..segments {
            let y = (i as f32 / segments as f32).powf(0.7);
            positions.push([-1.0, 0.0, y]);
            positions.push([1.0, 0.0, y]);
            uvs.push([0.0, y]);
            uvs.push([1.0, y]);

            if i > 0 {
                let base = 2 * (i - 1);
                indices.extend_from_slice(&[
                    base + 2, base + 1, base,
                    base + 3, base + 1, base + 2,
                ]);
            }
        }

        positions.push([0.0, 0.0, 1.0]);
        uvs.push([0.5, 1.0]);

        let tip = 2 * segments - 1;
        indices.extend_from_slice(&[
            tip, tip - 1, tip + 1,
        ]);

        let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        grass_mesh.insert_indices(Indices::U32(indices));

        grass_mesh.compute_normals();

        grass_mesh
    }
}