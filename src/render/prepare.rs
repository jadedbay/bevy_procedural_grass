use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer}, renderer::RenderDevice, texture::GpuImage, view::ViewUniforms}, utils::HashMap};
use super::pipeline::GrassComputePipeline;
use crate::{grass::{chunk::{GrassChunk, GrassChunkBuffers}, Grass, GrassGpuInfo},prefix_sum::{PrefixSumBindGroups, PrefixSumPipeline}};


// TODO: test whether this is actually improves performance or if its faster to recompute everyframe
#[derive(Resource, Default)]
pub struct ComputedGrassEntities(pub Vec<Entity>);

pub(crate) fn update_computed_grass(
    mut computed_grass: ResMut<ComputedGrassEntities>,
    q_chunks: Query<Entity, With<GrassChunk>>,
) {
    computed_grass.0.retain(|&entity| q_chunks.contains(entity));
}

#[derive(Component, Clone)]
pub struct GrassChunkBindGroups {
    pub chunk_bind_group: Option<BindGroup>,
    pub indirect_args_buffer: Buffer,

    pub cull_bind_group: BindGroup,

    pub cull_workgroup_count: u32,
    pub workgroup_count: u32,
    pub compact_workgroup_count: u32,

    pub compact_buffer: Buffer,
    pub compact_bind_group: BindGroup,

    pub reset_args_bind_group: BindGroup,
}


#[derive(Component)]
pub struct ComputeGrassMarker;

pub fn prepare_grass(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    prefix_sum_pipeline: Res<PrefixSumPipeline>,
    chunk_query: Query<(Entity, &GrassChunk, &GrassChunkBuffers)>,
    grass_query: Query<(&Grass, &GrassGpuInfo)>,
    computed_grass: Res<ComputedGrassEntities>,
    images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
) {
    let chunk_layout = pipeline.chunk_layout.clone();
    let cull_layout = pipeline.cull_layout.clone();
    let compact_layout = pipeline.compact_layout.clone();
    let reset_args_layout = pipeline.reset_args_layout.clone();

    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (entity, chunk, buffers) in chunk_query.iter() {
        let mut chunk_bind_group = None;

        let (grass, gpu_info) = grass_query.get(chunk.grass_entity).unwrap();

        if !computed_grass.0.contains(&entity) {
            chunk_bind_group = Some(render_device.create_bind_group(
                Some("buffers_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    buffers.instance_buffer.as_entire_binding(),
                    &images.get(grass.height_map.as_ref().unwrap().map.id()).unwrap().texture_view,
                    gpu_info.height_scale_buffer.as_entire_binding(),
                    gpu_info.height_offset_buffer.as_entire_binding(),
                    buffers.aabb_buffer.as_entire_binding(),
                    gpu_info.aabb_buffer.as_entire_binding(),
                )),
            ));
            commands.entity(entity).insert(ComputeGrassMarker);
        }
 
        let prefix_sum_bind_group = PrefixSumBindGroups::create_bind_groups(
            &render_device,
            &prefix_sum_pipeline,
            &buffers.vote_buffer,
            &buffers.prefix_sum_buffers,
            gpu_info.scan_workgroup_count,
            gpu_info.scan_groups_workgroup_count,
        );

        let cull_bind_group = render_device.create_bind_group(
            Some("cull_bind_group"),
            &cull_layout,
            &BindGroupEntries::sequential((
                buffers.instance_buffer.as_entire_binding(),
                buffers.vote_buffer.as_entire_binding(),
                view_uniform.clone(),
            ))
        );
        let indirect_indexed_args_buffer = &buffers.indirect_args_buffer; 

        let compact_bind_group = render_device.create_bind_group(
            Some("scan_bind_group"),
            &compact_layout,
            &BindGroupEntries::sequential((
                buffers.instance_buffer.as_entire_binding(),
                buffers.vote_buffer.as_entire_binding(),
                buffers.prefix_sum_buffers.scan_buffer.as_entire_binding(),
                buffers.prefix_sum_buffers.scan_blocks_out_buffer.as_entire_binding(),
                buffers.compact_buffer.as_entire_binding(),
                indirect_indexed_args_buffer.as_entire_binding(),
            )),
        );

        let reset_args_bind_group = render_device.create_bind_group(
            Some("reset_args_bind_group"),
            &reset_args_layout,
            &BindGroupEntries::single(indirect_indexed_args_buffer.as_entire_binding()),
        );

        let buffer_bind_group = GrassChunkBindGroups {
            chunk_bind_group,
            indirect_args_buffer: indirect_indexed_args_buffer.clone(),

            cull_bind_group,

            cull_workgroup_count: (gpu_info.instance_count as f32 / 256.).ceil() as u32,
            workgroup_count: gpu_info.workgroup_count,
            compact_workgroup_count: gpu_info.scan_workgroup_count,

            compact_buffer: buffers.compact_buffer.clone(),
            compact_bind_group,

            reset_args_bind_group,
        };

        commands.entity(entity).insert(buffer_bind_group);
        commands.entity(entity).insert(prefix_sum_bind_group);
    }
}