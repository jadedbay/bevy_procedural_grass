use bevy::{pbr::{MaterialExtension, MeshPipeline, MeshPipelineKey}, prelude::*, render::{mesh::MeshVertexBufferLayoutRef, render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_read_only_sized, storage_buffer_sized, texture_2d, uniform_buffer}, AsBindGroup, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, RenderPipelineDescriptor, ShaderStages, SpecializedMeshPipeline, SpecializedMeshPipelineError, TextureSampleType, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode}, renderer::RenderDevice, view::ViewUniform}};

use crate::{grass::chunk::Aabb2dGpu, GrassMaterial};

use super::instance::GrassInstanceData;

#[derive(Resource)]
pub(crate) struct GrassComputePipeline {
    pub chunk_layout: BindGroupLayout,
    pub cull_layout: BindGroupLayout,
    pub compact_layout: BindGroupLayout,
    pub reset_args_layout: BindGroupLayout,
    pub compute_id: CachedComputePipelineId,
    pub cull_pipeline_id: CachedComputePipelineId,
    pub compact_pipeline_id: CachedComputePipelineId,
    pub reset_args_pipeline_id: CachedComputePipelineId,

    _grass_types_shader: Handle<Shader>,
}

impl FromWorld for GrassComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let chunk_layout = render_device.create_bind_group_layout(
            "grass_chunk_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<Vec<GrassInstanceData>>(false),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    uniform_buffer::<f32>(false),
                    uniform_buffer::<f32>(false),
                    uniform_buffer::<Aabb2dGpu>(false), //TODO: dynamic offset
                    uniform_buffer::<Aabb2dGpu>(false),
                )
            )
        );

        let cull_layout = render_device.create_bind_group_layout(
            "cull_grass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer::<Vec<u32>>(false),
                    uniform_buffer::<ViewUniform>(true),
                )
            )
        );

        let compact_layout = render_device.create_bind_group_layout(
            "compact_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<GrassInstanceData>>(false),
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<GrassInstanceData>>(false),
                    storage_buffer_sized(false, None),
                )
            )
        );

        let reset_args_layout = render_device.create_bind_group_layout(
            "reset_args_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                storage_buffer_sized(false, None),
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compute_grass.wgsl");
        let cull_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_cull.wgsl");
        let compact_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compact.wgsl");
        let reset_args_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/reset_args.wgsl");
        
        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let compact_pipeline_id = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("compute_compact_grass_pipeline".into()),
                layout: vec![compact_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: compact_shader.clone(),
                shader_defs: vec![],
                entry_point: "compact".into(),
        });

        let cull_pipeline_id = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("cull_grass_pipeline".into()),
                layout: vec![cull_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: cull_shader.clone(),
                shader_defs: vec![],
                entry_point: "main".into(),
            }
        );

        let compute_id = pipeline_cache
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("grass_gen_compute_pipeline".into()),
                layout: vec![chunk_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: "main".into()
            });

        let reset_args_pipeline_id = pipeline_cache
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("reset_args_pipeline".into()),
                layout: vec![reset_args_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: reset_args_shader,
                shader_defs: vec![],
                entry_point: "reset_args".into(),
            });
        
        Self {
            chunk_layout,
            cull_layout,
            compact_layout,
            reset_args_layout,
            compute_id,
            cull_pipeline_id,
            compact_pipeline_id,
            reset_args_pipeline_id,
            _grass_types_shader: world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_types.wgsl")
        }
    }
}
