use std::time::Instant;

use bevy::{math::bounding::Aabb2d, prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, ShaderType}, renderer::RenderDevice}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::{prefix_sum::calculate_workgroup_counts, util::aabb::triangle_intersects_aabb};

#[derive(Component, Clone)]
pub struct GrassChunks(pub HashMap<UVec2, GrassChunk>);

#[derive(Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb2d,
    pub aabb_buffer: Buffer,

    pub instance_count: usize,
    pub workgroup_count: u32,
    pub scan_workgroup_count: u32,
    pub scan_groups_workgroup_count: u32,
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass)>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass) in grass_query.iter() {
        let mut grass_chunks = GrassChunks(HashMap::new());

        let mesh = meshes.get(ground_query.get(grass.ground_entity.unwrap()).unwrap()).unwrap();
        let mesh_aabb = mesh.compute_aabb().unwrap();
        let mesh_size = mesh_aabb.max() - mesh_aabb.min();
        let chunk_count = (mesh_size / grass.chunk_size).ceil().max(Vec3::splat(1.0).into());

        for x in 0..chunk_count.x as usize {
            for z in 0..chunk_count.z as usize {
                let min = Vec2::new(
                    mesh_aabb.min().x + grass.chunk_size * x as f32,
                    mesh_aabb.min().z + grass.chunk_size * z as f32
                );
                let max = min + Vec2::splat(grass.chunk_size);
                let aabb = Aabb2d::new(min, max);

                grass_chunks.0.insert(
                    UVec2::new(x as u32, z as u32), 
                    GrassChunk {
                        aabb,
                        aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("aabb_buffer"),
                            contents: bytemuck::cast_slice(&[Aabb2dGpu::from(aabb)]),
                            usage: BufferUsages::UNIFORM,
                        }),
                        instance_count: 0,
                        workgroup_count: 0,
                        scan_groups_workgroup_count: 0,
                        scan_workgroup_count: 0,
                    }
                );
            }
        }

        // Calculate workgroup counts
        for (_, chunk) in grass_chunks.0.iter_mut() {
            let workgroup_count = grass.density as usize;
            let instance_count = workgroup_count * 512;

            dbg!(instance_count);

            let (scan_workgroup_count, scan_groups_workgroup_count) = calculate_workgroup_counts(instance_count as u32);

            chunk.instance_count = instance_count;
            chunk.workgroup_count = workgroup_count as u32;
            chunk.scan_workgroup_count = scan_workgroup_count;
            chunk.scan_groups_workgroup_count = scan_groups_workgroup_count;
        }

        commands.entity(entity).insert(grass_chunks);
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub(crate) struct Aabb2dGpu {
    min: Vec2,
    max: Vec2,
}

impl From<Aabb2d> for Aabb2dGpu {
    fn from(aabb: Aabb2d) -> Self {
        Self {
            min: aabb.min,
            max: aabb.max,
        }
    }
}
