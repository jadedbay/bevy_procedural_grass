use std::time::Instant;

use bevy::{math::bounding::Aabb2d, prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::Aabb, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, ShaderType}, renderer::RenderDevice}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::{prefix_sum::calculate_workgroup_counts, render::instance::GrassInstanceData, util::aabb::triangle_intersects_aabb};

#[derive(Component, Clone)]
pub struct GrassChunks(pub HashMap<UVec2, GrassChunk>);

#[derive(Component, Clone)]
pub struct GrassAabb {
    pub buffer: Buffer,
}

#[derive(Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb2d,
    pub aabb_buffer: Buffer,
    pub instance_buffer: Buffer,

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
        let mesh_aabb2d = Aabb2d::new(mesh_aabb.center.xz(), mesh_aabb.half_extents.xz());

        let aabb_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("aabb_buffer"),
            contents: bytemuck::cast_slice(&[Aabb2dGpu::from(mesh_aabb2d)]),
            usage: BufferUsages::UNIFORM
        });

        let chunk_size = (mesh_aabb2d.max - mesh_aabb2d.min) / (grass.chunk_count.as_vec2());

        let workgroup_count = (grass.density as f32 / (grass.chunk_count.x * grass.chunk_count.y) as f32).ceil() as usize;
        let instance_count = workgroup_count * 512;
        let (scan_workgroup_count, scan_groups_workgroup_count) = calculate_workgroup_counts(instance_count as u32);

        for x in 0..grass.chunk_count.x { 
            for z in 0..grass.chunk_count.y {
                let chunk_min = mesh_aabb2d.min + Vec2::new(x as f32, z as f32) * chunk_size;
                let chunk_max = chunk_min + chunk_size; 
                let aabb = Aabb2d {
                    min: chunk_min,
                    max: chunk_max,
                };

                grass_chunks.0.insert(
                    UVec2::new(x as u32, z as u32), 
                    GrassChunk {
                        aabb,
                        aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("aabb_buffer"),
                            contents: bytemuck::cast_slice(&[Aabb2dGpu::from(aabb)]),
                            usage: BufferUsages::UNIFORM,
                        }),
                        instance_buffer: render_device.create_buffer(&BufferDescriptor {
                            label: Some("instance_buffer"),
                            size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                            usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                            mapped_at_creation: false, 
                        }),                      
                        instance_count,
                        workgroup_count: workgroup_count as u32,
                        scan_groups_workgroup_count,
                        scan_workgroup_count,
                    }
                );
            }
        }

        commands.entity(entity).insert(grass_chunks);
        commands.entity(entity).insert(GrassAabb {
            buffer: aabb_buffer
        });
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
