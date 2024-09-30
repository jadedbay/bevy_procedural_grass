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
    chunk::{Aabb2dGpu, GrassChunks}, Grass, GrassGpuInfo},
prefix_sum::{create_prefix_sum_bind_group_buffers, PrefixSumBindGroup, PrefixSumPipeline}};

#[derive(Resource, Default)]
pub struct GrassEntities(pub HashMap<Entity, GrassStage>);

#[derive(Clone, Copy, Default, PartialEq)]
pub enum GrassStage {
    #[default]
    Compute,
    Cull,
}

#[derive(Clone)]
pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: Option<BindGroup>,
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
    query: Query<(Entity, &GrassChunks, &Grass, &GrassGpuInfo)>,
    grass_entities: Res<GrassEntities>,
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

    for (entity, chunks, grass, gpu_info) in query.iter() { 
        let mut chunk_bind_groups = Vec::new();
        let mut prefix_sum_bind_groups = Vec::new();

        let aabb_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("aabb_buffer"),
            contents: bytemuck::cast_slice(&[Aabb2dGpu::from(gpu_info.aabb)]),
            usage: BufferUsages::UNIFORM,
        });

        let height_scale_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("height_scale_buffer"),
                contents: bytemuck::cast_slice(&[grass.height_map.as_ref().unwrap().scale]),
                usage: BufferUsages::UNIFORM,
            }
        );
        let height_offset_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("height_offset_buffer"),
                contents: bytemuck::cast_slice(&[grass.y_offset]),
                usage: BufferUsages::UNIFORM,
            }
        );

        for (_, (chunk, in_frustum)) in chunks.0.clone().into_iter() {
            if !in_frustum { continue; }

            let mut chunk_bind_group = None;

            if !grass_entities.0.contains_key(&entity) {
                chunk_bind_group = Some(render_device.create_bind_group(
                    Some("chunk_bind_group"),
                    &chunk_layout,
                    &BindGroupEntries::sequential((
                        chunk.instance_buffer.as_entire_binding(),
                        &images.get(grass.height_map.as_ref().unwrap().map.id()).unwrap().texture_view,
                        height_scale_buffer.as_entire_binding(),
                        height_offset_buffer.as_entire_binding(),
                        chunk.aabb_buffer.as_entire_binding(),
                        aabb_buffer.as_entire_binding(),
                    )),
                ));
            }

            let vote_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("vote_buffer"),
                size: (std::mem::size_of::<u32>() * gpu_info.instance_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            let compact_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("compact_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * gpu_info.instance_count) as u64,
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
                gpu_info.instance_count as u32,
                gpu_info.scan_workgroup_count,
                gpu_info.scan_groups_workgroup_count,
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

                cull_workgroup_count: (gpu_info.instance_count as f32 / 256.).ceil() as u32,
                workgroup_count: gpu_info.workgroup_count,
                compact_workgroup_count: gpu_info.scan_workgroup_count,

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