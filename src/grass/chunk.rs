use std::time::Instant;

use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb, render_resource::ShaderType}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::util::aabb::triangle_intersects_aabb;

#[derive(Component, Clone)]
pub struct GrassChunks(pub HashMap<UVec3, GrassChunk>);

#[derive(Debug, Clone, Default)]
pub struct GrassChunk {
    pub aabb: Aabb,
    pub indices_index: Vec<u32>,

    pub instance_count: usize,
    pub workgroup_count: usize,
    pub scan_workgroup_count: usize,
    pub scan_groups_workgroup_count: usize,
}

#[derive(Clone, Component)]
pub struct GrassChunksP {
    pub aabbs: Vec<Aabb>,
    pub chunks: UVec3,
    pub triangle_count: usize,
}

impl GrassChunksP {
    pub fn chunk_count(&self) -> u32 {
        self.chunks.x * self.chunks.y * self.chunks.z
    }
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass)>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
) {
    for (entity, grass) in grass_query.iter() {
        let start = Instant::now();
        let mut grass_chunks = GrassChunks(HashMap::new());
        let mut aabbs = Vec::new();

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
                            ..default()
                        }
                    );

                    aabbs.push(aabb);
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

        commands.entity(entity).insert(GrassChunksP {
            aabbs,
            chunks: UVec3::new(chunk_count.x as u32, chunk_count.y as u32, chunk_count.z as u32),
            triangle_count: (indices.len() / 3), // NOTE: triangle count for entire mesh (not just
            // chunk)
        });

        // Add indices to chunk
        for (i, triangle) in indices.chunks(3).enumerate() {
            let v0 = Vec3::from(positions[triangle[0] as usize]);
            let v1 = Vec3::from(positions[triangle[1] as usize]);
            let v2 = Vec3::from(positions[triangle[2] as usize]);

            // let density = 0.5; // TODO
            // let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
            // let blade_count = (density * area).ceil() as u32;

            for (_, chunk) in grass_chunks.0.iter_mut() {
                if triangle_intersects_aabb(v0, v1, v2, &chunk.aabb) {
                    //for _ in 0..(blade_count as f32 / 8.0).ceil() as u32 {
                        chunk.indices_index.push(i as u32);
                    //}
                }
            }
        }

        // Calculate workgroup counts
        for (_, chunk) in grass_chunks.0.iter_mut() {
            let workgroup_count = chunk.indices_index.len();
            let instance_count = workgroup_count * 32;

            dbg!(instance_count);

            let mut scan_workgroup_count = (instance_count as f32 / 128.).ceil() as u32;
            if scan_workgroup_count > 128 {
                let mut p2 = 128;
                while p2 < scan_workgroup_count {
                    p2 *= 2;
                }

                scan_workgroup_count = p2;
            } else {
                while 128 % scan_workgroup_count != 0 {
                    scan_workgroup_count += 1;
                }
            }

            let scan_groups_workgroup_count = (instance_count as f32 / 1024.).ceil() as u32;

            chunk.instance_count = instance_count;
            chunk.workgroup_count = workgroup_count;
            chunk.scan_workgroup_count = scan_workgroup_count as usize;
            chunk.scan_groups_workgroup_count = scan_groups_workgroup_count as usize;
        }

        commands.entity(entity).insert(grass_chunks);

        dbg!(start.elapsed());
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
