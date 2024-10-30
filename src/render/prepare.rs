use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, DynamicBindGroupEntries, PipelineCache, SpecializedComputePipelines}, renderer::RenderDevice, texture::GpuImage, view::ViewUniforms}};
use super::pipeline::{GrassComputePipeline, GrassCullPipeline, GrassCullPipelineId};
use crate::{grass::{chunk::{GrassChunk, GrassChunkBuffers, GrassChunkCullBuffers}, config::GrassConfigBuffer, Grass, GrassGpuInfo},prefix_sum::{PrefixSumBindGroups, PrefixSumPipeline}, prelude::GrassLODMesh};


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
pub struct GrassChunkComputeBindGroup {
    pub bind_group: BindGroup,
    pub workgroup_count: u32,
}

#[derive(Component, Clone)]
pub struct CompactBindGroupsLOD(pub(crate) CompactBindGroups);
#[derive(Component, Clone)]
pub struct PrefixSumBindGroupsLOD(pub(crate) PrefixSumBindGroups);

#[derive(Component, Clone)]
pub struct GrassChunkCullBindGroup {
    pub cull_workgroup_count: u32,
    pub cull_bind_group: BindGroup,
}
impl GrassChunkCullBindGroup {
    fn create_bind_group(
        render_device: &RenderDevice,
        gpu_info: &GrassGpuInfo,
        buffers: &GrassChunkBuffers,
        view_uniforms: &ViewUniforms,
        config_buffers: &GrassConfigBuffer,
        cull_pipeline: &GrassCullPipeline,
    ) -> Self {
        let mut entries = DynamicBindGroupEntries::sequential((
            buffers.instance_buffer.as_entire_binding(),
            buffers.compact_buffers.vote_buffer.as_entire_binding(),
            view_uniforms.uniforms.binding().unwrap().clone(),
            config_buffers.0.as_entire_binding(),
        ));

        let (layout, name) = if let Some(lod_buffers) = &buffers.lod_compact_buffers {
            entries = entries.extend_sequential((lod_buffers.vote_buffer.as_entire_binding(),));
            
            if let Some(shadow_buffers) = &buffers.shadow_compact_buffers {
                entries = entries.extend_sequential((shadow_buffers.vote_buffer.as_entire_binding(),));
                (&cull_pipeline.cull_layout_shadow_lod, "cull_bind_group_with_shadows_lod")
            } else {
                (&cull_pipeline.cull_layout_or, "cull_bind_group_lod")
            }
        } else if let Some(shadow_buffers) = &buffers.shadow_compact_buffers {
            entries = entries.extend_sequential((shadow_buffers.vote_buffer.as_entire_binding(),));
            (&cull_pipeline.cull_layout_or, "cull_bind_group_with_shadows")
        } else {
            (&cull_pipeline.cull_layout, "cull_bind_group")
        };

        let cull_bind_group = render_device.create_bind_group(
            Some(name),
            layout,
            &entries,
        );

        Self {
            cull_workgroup_count: (gpu_info.instance_count as f32 / 256.).ceil() as u32,
            cull_bind_group,
        }
    }
}

#[derive(Component, Clone)]
pub struct CompactBindGroups {
    pub indirect_args_buffer: Buffer,

    pub compact_workgroup_count: u32,
    pub compact_buffer: Buffer,
    pub compact_bind_group: BindGroup,

    pub reset_args_bind_group: BindGroup,
}
impl CompactBindGroups {
    fn create_bind_groups(
        render_device: &RenderDevice,
        buffers: &GrassChunkBuffers,
        compact_buffers: &GrassChunkCullBuffers,
        gpu_info: &GrassGpuInfo,
        pipeline: &GrassComputePipeline,
    ) -> Self {
        let indirect_indexed_args_buffer = &compact_buffers.indirect_args_buffer; 

        let compact_bind_group = render_device.create_bind_group(
            Some("scan_bind_group"),
            &pipeline.compact_layout,
            &BindGroupEntries::sequential((
                buffers.instance_buffer.as_entire_binding(),
                compact_buffers.vote_buffer.as_entire_binding(),
                compact_buffers.prefix_sum_buffers.scan_buffer.as_entire_binding(),
                compact_buffers.prefix_sum_buffers.scan_blocks_out_buffer.as_entire_binding(),
                compact_buffers.compact_buffer.as_entire_binding(),
                indirect_indexed_args_buffer.as_entire_binding(),
            )),
        );

        let reset_args_bind_group = render_device.create_bind_group(
            Some("reset_args_bind_group"),
            &pipeline.reset_args_layout,
            &BindGroupEntries::single(indirect_indexed_args_buffer.as_entire_binding()),
        );

        Self {
            indirect_args_buffer: indirect_indexed_args_buffer.clone(),
            compact_workgroup_count: gpu_info.scan_workgroup_count,
            compact_buffer: compact_buffers.compact_buffer.clone(),
            compact_bind_group,
            reset_args_bind_group,
        }
    }
}

#[derive(Component, Clone)]
pub struct GrassShadowBindGroups(pub CompactBindGroups);

#[derive(Component, Clone)]
pub struct ShadowPrefixSumBindGroups(pub PrefixSumBindGroups);

pub fn prepare_grass(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    cull_pipeline: Res<GrassCullPipeline>,
    prefix_sum_pipeline: Res<PrefixSumPipeline>,
    chunk_query: Query<(Entity, &GrassChunk, &GrassChunkBuffers)>,
    grass_query: Query<(&Grass, &GrassGpuInfo)>,
    computed_grass: Res<ComputedGrassEntities>,
    images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    grass_config_buffers: Res<GrassConfigBuffer>,
) {
    let Some(_) = view_uniforms.uniforms.binding() else { return; };
    let chunk_layout = pipeline.chunk_layout.clone();

    for (entity, chunk, buffers) in chunk_query.iter() {
        let (grass, gpu_info) = grass_query.get(chunk.grass_entity).unwrap();

        if !computed_grass.0.contains(&entity) {
            let chunk_bind_group = render_device.create_bind_group(
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
            );
            commands.entity(entity).insert(GrassChunkComputeBindGroup {
                bind_group: chunk_bind_group,
                workgroup_count: gpu_info.workgroup_count,
            });
        }

        commands.entity(entity).insert(
            GrassChunkCullBindGroup::create_bind_group(
                &render_device,
                gpu_info,
                buffers,
                &view_uniforms,
                &grass_config_buffers,
                &cull_pipeline,
            )
        );

        commands.entity(entity).insert(
            CompactBindGroups::create_bind_groups(
                &render_device,
                buffers,
                &buffers.compact_buffers,
                gpu_info,
                &pipeline,
            )
        );
        commands.entity(entity).insert(
            PrefixSumBindGroups::create_bind_groups(
                &render_device,
                &prefix_sum_pipeline,
                &buffers.compact_buffers.vote_buffer,
                &buffers.compact_buffers.prefix_sum_buffers,
                gpu_info.scan_workgroup_count,
                gpu_info.scan_groups_workgroup_count,
            )
        );

        if let Some(lod_buffers) = &buffers.lod_compact_buffers {
            commands.entity(entity).insert((
                CompactBindGroupsLOD(
                    CompactBindGroups::create_bind_groups(
                        &render_device,
                        buffers,
                        &lod_buffers,
                        gpu_info,
                        &pipeline,
                    )
                ),
                PrefixSumBindGroupsLOD(
                    PrefixSumBindGroups::create_bind_groups(
                        &render_device,
                        &prefix_sum_pipeline,
                        &lod_buffers.vote_buffer,
                        &lod_buffers.prefix_sum_buffers,
                        gpu_info.scan_workgroup_count,
                        gpu_info.scan_groups_workgroup_count,
                    )
                )
            ));
        }

        if let Some(shadow_buffers) = &buffers.shadow_compact_buffers {
            commands.entity(entity).insert((
                GrassShadowBindGroups(
                    CompactBindGroups::create_bind_groups(
                        &render_device,
                        buffers,
                        &shadow_buffers,
                        gpu_info,
                        &pipeline,
                    )
                ),
                ShadowPrefixSumBindGroups(
                    PrefixSumBindGroups::create_bind_groups(
                        &render_device,
                        &prefix_sum_pipeline,
                        &shadow_buffers.vote_buffer,
                        &shadow_buffers.prefix_sum_buffers,
                        gpu_info.scan_workgroup_count,
                        gpu_info.scan_groups_workgroup_count,
                    )
                )
            ));
        }
    }
}
