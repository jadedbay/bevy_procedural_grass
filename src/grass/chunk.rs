use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb}, utils::HashMap};
use super::Grass;
use crate::util::aabb::triangle_intersects_aabb;

pub(super) type GrassChunks = HashMap<UVec2, GrassChunk>;

#[derive(Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb,
    pub(crate) mesh_indices: Vec<u32>,
}

pub(crate) fn create_chunks(
    meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut Grass, &Handle<Mesh>)>,
) {
    for (mut grass, mesh_handle) in query.iter_mut() {
        let mesh = meshes.get(mesh_handle).unwrap();
        let mesh_aabb = mesh.compute_aabb().unwrap();
        let mesh_size = mesh_aabb.max() - mesh_aabb.min();
        let chunk_count = (mesh_size / grass.chunk_size).ceil();

        grass.chunk_count = UVec2::new(chunk_count.x as u32, chunk_count.z as u32);

        for x in 0..chunk_count.x as usize {
            for z in 0..chunk_count.z as usize {
                let min = Vec3::from(mesh_aabb.min()) + Vec3::new(grass.chunk_size * x as f32, 0.0, grass.chunk_size * z as f32);
                let max = min + Vec3::splat(grass.chunk_size);
                let aabb = Aabb::from_min_max(min, max);

                grass.chunks.insert(
                    UVec2::new(x as u32, z as u32), 
                    GrassChunk {
                        aabb,
                        mesh_indices: Vec::new(),
                    }
                );
            }
        }

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => {
                warn!("Mesh does not contain positions, not generating grass.");
                return;
            },
        };

        // TODO: allow meshes that dont have indices
        let indices = match mesh.indices() {
            Some(Indices::U32(indices)) => indices,
            _ => {
                warn!("Mesh does not contain indices, not generating grass.");
                return;
            }, 
        };

        for triangle in indices.chunks(3) {
            let v0 = Vec3::from(positions[triangle[0] as usize]);
            let v1 = Vec3::from(positions[triangle[1] as usize]);
            let v2 = Vec3::from(positions[triangle[2] as usize]);

            for (_, chunk) in grass.chunks.iter_mut() {
                if triangle_intersects_aabb(v0, v1, v2, &chunk.aabb) {
                    chunk.mesh_indices.extend_from_slice(triangle);
                }
            }
        }
    }
}

