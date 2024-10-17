
use bevy::{ecs::query::QueryItem, math::bounding::{Aabb2d, BoundingVolume}, prelude::*, render::{extract_component::ExtractComponent, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, ShaderType}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};
use crate::{grass::{cull::GrassCullChunks, GrassGpuInfo}, prefix_sum::PrefixSumBuffers, render::instance::GrassInstanceData, util::aabb::Aabb2dGpu, GrassMaterial};

#[derive(Component, Clone)]
pub struct GrassChunk {
    pub grass_entity: Entity,
    pub aabb: Aabb2d,

    pub instance_count: usize,
    pub scan_workgroup_count: u32,
}

#[derive(Component, Clone)]
pub struct GrassChunkBuffers {
    pub aabb_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub cull_buffers: GrassChunkCullBuffers,
    pub(crate) shadow_buffers: Option<GrassChunkCullBuffers>,
}

#[derive(Clone)]
pub struct GrassChunkCullBuffers {
    pub vote_buffer: Buffer,
    pub compact_buffer: Buffer,
    pub indirect_args_buffer: Buffer,
    pub(crate) prefix_sum_buffers: PrefixSumBuffers,
}
impl GrassChunkCullBuffers {
    fn create_buffers(
        render_device: &RenderDevice,
        instance_count: usize,
        scan_workgroup_count: u32,
    ) -> Self {
        Self {
            vote_buffer: render_device.create_buffer(&BufferDescriptor {
                label: Some("vote_buffer"),
                size: (std::mem::size_of::<u32>() * instance_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            compact_buffer: render_device.create_buffer(&BufferDescriptor {
                label: Some("compact_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            indirect_args_buffer: render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indirect_indexed_args"),
                    contents: DrawIndexedIndirectArgs {
                        index_count: 39, // TODO
                        instance_count: 0,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: 0,
                    }.as_bytes(),
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            }),
            prefix_sum_buffers: PrefixSumBuffers::create_buffers(
                &render_device, 
                instance_count as u32, 
                scan_workgroup_count,
            ),
        }
    }
}

impl GrassChunkBuffers {
    pub(crate) fn create_buffers(
        render_device: &RenderDevice,
        aabb: Aabb2d,
        instance_count: usize,
        scan_workgroup_count: u32,
        grass_shadows: bool,
    ) -> Self {
        let cull_buffers = GrassChunkCullBuffers::create_buffers(render_device, instance_count, scan_workgroup_count);
        let shadow_buffers = if grass_shadows {
            Some(GrassChunkCullBuffers::create_buffers(render_device, instance_count, scan_workgroup_count))
        } else {
            None
        };

        Self {
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
            cull_buffers,
            shadow_buffers,
        }
    }
}

impl ExtractComponent for GrassChunk {
    type QueryData = (&'static GrassChunk, &'static GrassChunkBuffers);
    type QueryFilter = ();
    type Out = (GrassChunk, GrassChunkBuffers);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}

pub(crate) fn unload_chunks(
    commands: &mut Commands,
    entity: Entity,
    cull_chunks: &mut GrassCullChunks,
) {
    for (_, chunk_entity) in cull_chunks.0.iter() {
        commands.entity(entity).remove_children(&[*chunk_entity]);
        commands.entity(*chunk_entity).despawn();
    }
    cull_chunks.0.clear();
}