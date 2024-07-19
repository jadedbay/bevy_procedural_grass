use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb, render_resource::ShaderType}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::util::aabb::triangle_intersects_aabb;

#[derive(Component, Clone)]
pub struct GrassChunks(pub HashMap<UVec3, GrassChunk>);

#[derive(Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb,
    pub indices_index: Vec<u32>,
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass)>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
) {
    for (entity, grass) in grass_query.iter() {
        let mut grass_chunks = GrassChunks(HashMap::new());

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

                    grass_chunks.0.insert(
                        UVec3::new(x as u32, y as u32, z as u32), 
                        GrassChunk {
                            aabb,
                            indices_index: Vec::new(),
                        }
                    );
                }
            }
        }

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => {
                warn!("Mesh does not contain positions");
                return;
            },
        };

        let indices = match mesh.indices() {
            Some(Indices::U32(indices)) => indices,
            _ => {
                warn!("Mesh does not contain indices");
                return;
            }, 
        };

        for (i, triangle) in indices.chunks(3).enumerate() {
            let v0 = Vec3::from(positions[triangle[0] as usize]);
            let v1 = Vec3::from(positions[triangle[1] as usize]);
            let v2 = Vec3::from(positions[triangle[2] as usize]);

            let density = 12.0; // TODO
            let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
            let blade_count = (density * area).ceil() as u32;
            let dispatch_count = (blade_count as f32 / 8.0).ceil() as u32;

            for (_, chunk) in grass_chunks.0.iter_mut() {
                if triangle_intersects_aabb(v0, v1, v2, &chunk.aabb) {
                    for _ in 0..dispatch_count {
                        chunk.indices_index.push(i as u32);
                    }
                }
            }
        }

        commands.entity(entity).insert(grass_chunks);
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub(crate) struct BoundingBox {
    min: Vec3,
    _padding: f32,
    max: Vec3,
    _padding2: f32,
}

impl From<Aabb> for BoundingBox {
    fn from(aabb: Aabb) -> Self {
        Self {
            min: aabb.min().into(),
            _padding: 0.0,
            max: aabb.max().into(),
            _padding2: 0.0,
        }
    }
}