use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::util::aabb::triangle_intersects_aabb;

pub(super) type GrassChunks = HashMap<UVec3, GrassChunk>;

#[derive(Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb,
    pub mesh_indices: Vec<u32>,
    pub indices_index: Vec<u32>,
}

pub(crate) fn create_chunks(
    meshes: ResMut<Assets<Mesh>>,
    mut grass_query: Query<&mut Grass>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
) {
    for mut grass in grass_query.iter_mut() {
        let mesh = meshes.get(ground_query.get(grass.ground_entity.unwrap()).unwrap()).unwrap();
        let mesh_aabb = mesh.compute_aabb().unwrap();
        let mesh_size = mesh_aabb.max() - mesh_aabb.min();
        let chunk_count = (mesh_size / grass.chunk_size).ceil().max(Vec3::splat(1.0).into());

        for x in 0..chunk_count.x as usize {
            for y in 0..chunk_count.y as usize {
                for z in 0..chunk_count.z as usize {
                    let min = Vec3::from(mesh_aabb.min()) + Vec3::new(grass.chunk_size * x as f32, grass.chunk_size * y as f32, grass.chunk_size * z as f32);
                    let max = min + Vec3::splat(grass.chunk_size);
                    let aabb = Aabb::from_min_max(min, max);

                    grass.chunks.insert(
                        UVec3::new(x as u32, y as u32, z as u32), 
                        GrassChunk {
                            aabb,
                            mesh_indices: Vec::new(),
                            indices_index: Vec::new(),
                        }
                    );
                }
            }
        }

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => {
                warn!("Mesh does not contain positions, not generating grass.");
                return;
            },
        };

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

            let density = 4.0; // TODO
            let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
            let blade_count = (density * area).ceil() as u32;

            for (_, chunk) in grass.chunks.iter_mut() {
                if triangle_intersects_aabb(v0, v1, v2, &chunk.aabb) {
                    let index = chunk.mesh_indices.len() / 3;

                    chunk.mesh_indices.extend_from_slice(triangle);
                    
                    let dispatch_count = (blade_count as f32 / 8.0).ceil() as u32;
                    for _ in 0..dispatch_count {
                        chunk.indices_index.push(index as u32);
                    }
                }
            }
        }

        for (_, chunk) in &grass.chunks {
            dbg!(&chunk.indices_index);
        }
    }
}

