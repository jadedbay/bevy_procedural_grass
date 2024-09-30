use bevy::{pbr::{MeshPipeline, MeshPipelineKey}, prelude::*, render::{mesh::MeshVertexBufferLayoutRef, render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_read_only_sized, storage_buffer_sized, texture_2d, uniform_buffer}, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, PushConstantRange, RenderPipelineDescriptor, ShaderDefVal, ShaderStages, SpecializedComputePipeline, SpecializedMeshPipeline, SpecializedMeshPipelineError, TextureSampleType, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode}, renderer::RenderDevice, view::ViewUniform}};

use crate::grass::chunk::Aabb2dGpu;

use super::instance::GrassInstanceData;

#[derive(Resource)]
pub(crate) struct GrassComputePipeline {
    pub chunk_layout: BindGroupLayout,
    pub cull_layout: BindGroupLayout,
    pub compact_layout: BindGroupLayout,
    pub compute_id: CachedComputePipelineId,
    pub cull_pipeline_id: CachedComputePipelineId,
    pub compact_pipeline_id: CachedComputePipelineId,

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

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compute_grass.wgsl");
        let cull_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_cull.wgsl");
        let compact_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compact.wgsl");

        
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
        
        Self {
            chunk_layout,
            cull_layout,
            compact_layout,
            compute_id,
            cull_pipeline_id,
            compact_pipeline_id,
            _grass_types_shader: world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_types.wgsl")
        }
    }
}

#[derive(Resource)]
pub(crate) struct GrassRenderPipeline {
    shader: Handle<Shader>,
    pub mesh_pipeline: MeshPipeline,
}

impl FromWorld for GrassRenderPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>();

        GrassRenderPipeline {
            shader: world.load_asset("embedded://bevy_procedural_grass/shaders/grass.wgsl"),
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for GrassRenderPipeline {
    type Key = MeshPipelineKey;
    
    fn specialize(
            &self,
            key: Self::Key,
            layout: &MeshVertexBufferLayoutRef,
        ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
            let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

            descriptor.vertex.shader = self.shader.clone();
            descriptor.vertex.buffers.push(VertexBufferLayout {
                array_stride: std::mem::size_of::<GrassInstanceData>() as u64,
                step_mode: VertexStepMode::Instance,
                attributes: vec![
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 3,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: std::mem::size_of::<[f32; 4]>() as u64,
                        shader_location: 4,
                    },
                ],
            });
            descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
            descriptor.primitive.cull_mode = None;
            Ok(descriptor)
    }
}
