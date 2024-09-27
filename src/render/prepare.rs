use bevy::{
    ecs::world, gizmos::aabb, prelude::*, render::{
        render_asset::RenderAssets, render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindingResource, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, PipelineCache, SpecializedComputePipelines
        }, renderer::RenderDevice, texture::GpuImage, view::ViewUniforms
    }, utils::HashMap
};

use super::{
    instance::GrassInstanceData,
    pipeline::GrassComputePipeline,
};
use crate::{grass::{
    chunk::{Aabb2dGpu, GrassAabb, GrassChunks}, Grass},
prefix_sum::{create_prefix_sum_bind_group_buffers, PrefixSumBindGroup, PrefixSumPipeline}};

#[derive(Resource, Default)]
pub struct GrassEntities(pub HashMap<Entity, GrassStage>);

pub struct GrassEntityPersistentBuffers {
    pub indices_index_buffers: Option<Vec<Buffer>>,
    pub instances_buffers: Option<Vec<Buffer>>,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum GrassStage {
    #[default]
    Compute,
    Cull,
}

#[derive(Clone)]
pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub indirect_args_buffer: Buffer,

    pub cull_bind_group: BindGroup,

    pub cull_workgroup_count: u32,
    pub workgroup_count: u32,
    pub compact_workgroup_count: u32,

    pub compact_buffer: Buffer,
    pub compact_bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct GrassBufferBindGroup {
    pub chunks: Vec<GrassChunkBufferBindGroup>,
    pub prefix_sum_chunks: Vec<PrefixSumBindGroup>,
}


pub fn prepare_grass(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    prefix_sum_pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &GrassChunks, &Grass, &GrassAabb)>,
    images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
) {
    let chunk_layout = pipeline.chunk_layout.clone();
    let cull_layout = pipeline.cull_layout.clone();
    let compact_layout = pipeline.compact_layout.clone();

    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (entity, chunks, grass, aabb) in query.iter() { 
        let mut chunk_bind_groups = Vec::new();
        let mut prefix_sum_bind_groups = Vec::new();

        for (_, chunk) in chunks.0.clone().into_iter() {

            let vote_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("vote_buffer"),
                size: (std::mem::size_of::<u32>() * chunk.instance_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            let chunk_bind_group = render_device.create_bind_group(
                Some("chunk_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    chunk.instance_buffer.as_entire_binding(),
                    &images.get(grass.height_map.as_ref().unwrap().id()).unwrap().texture_view,
                    chunk.aabb_buffer.as_entire_binding(),
                    aabb.buffer.as_entire_binding(),
                )),
            );

            let compact_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("compact_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * chunk.instance_count) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            let indirect_indexed_args_buffer =
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("indirect_indexed_args"),
                    contents: DrawIndexedIndirectArgs {
                        index_count: 39, // TODO
                        instance_count: 0,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: 0,
                    }.as_bytes(),
                    usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
                });
            
            let prefix_sum_bind_group = create_prefix_sum_bind_group_buffers(
                &render_device,
                &prefix_sum_pipeline,
                &vote_buffer,
                chunk.instance_count as u32,
                chunk.scan_workgroup_count,
                chunk.scan_groups_workgroup_count,
            );

            let cull_bind_group = render_device.create_bind_group(
                Some("cull_bind_group"),
                &cull_layout,
                &BindGroupEntries::sequential((
                    chunk.instance_buffer.as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    view_uniform.clone(),
                ))
            );

            let compact_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &compact_layout,
                &BindGroupEntries::sequential((
                    chunk.instance_buffer.as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    prefix_sum_bind_group.scan_buffer.as_entire_binding(),
                    prefix_sum_bind_group.scan_blocks_out_buffer.as_entire_binding(),
                    compact_buffer.as_entire_binding(),
                    indirect_indexed_args_buffer.as_entire_binding(),
                )),
            );

            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                indirect_args_buffer: indirect_indexed_args_buffer,

                cull_bind_group,

                cull_workgroup_count: (chunk.instance_count as f32 / 256.).ceil() as u32,
                workgroup_count: chunk.workgroup_count,
                compact_workgroup_count: chunk.scan_workgroup_count,

                compact_buffer,
                compact_bind_group,
            });

            prefix_sum_bind_groups.push(prefix_sum_bind_group);
        }

        let buffer_bind_group = GrassBufferBindGroup {
            chunks: chunk_bind_groups,
            prefix_sum_chunks: prefix_sum_bind_groups,
        };

        commands.entity(entity).insert(buffer_bind_group.clone());
    }
}