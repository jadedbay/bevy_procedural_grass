
use bevy::{ecs::query::QueryItem, math::bounding::{Aabb2d, BoundingVolume}, prelude::*, render::{extract_component::ExtractComponent, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, ShaderType}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};
use super::Grass;
use crate::{grass::{cull::GrassCullChunks, GrassGpuInfo}, prefix_sum::{calculate_workgroup_counts, PrefixSumBuffers}, render::instance::GrassInstanceData, GrassMaterial};

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
    pub shadow_buffers: GrassShadowBuffers,
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

#[derive(Clone)]
pub(crate) struct GrassShadowBuffers(pub GrassChunkCullBuffers);

impl GrassChunkBuffers {
    pub(crate) fn create_buffers(
        render_device: &RenderDevice,
        aabb: Aabb2d,
        instance_count: usize,
        scan_workgroup_count: u32,
    ) -> Self {
        let cull_buffers = GrassChunkCullBuffers::create_buffers(render_device, instance_count, scan_workgroup_count);
        let shadow_buffers = GrassShadowBuffers(GrassChunkCullBuffers::create_buffers(render_device, instance_count, scan_workgroup_count));

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
