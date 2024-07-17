use bevy::{pbr::{MeshPipeline, MeshPipelineKey}, prelude::*, render::{mesh::MeshVertexBufferLayoutRef, render_resource::{binding_types::{storage_buffer, storage_buffer_read_only}, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, RenderPipelineDescriptor, ShaderStages, SpecializedMeshPipeline, SpecializedMeshPipelineError, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode}, renderer::RenderDevice}};

use super::instance::GrassInstanceData;

#[derive(Resource)]
pub(crate) struct GrassComputePipeline {
    pub mesh_layout: BindGroupLayout,
    pub chunk_layout: BindGroupLayout,
    pub compute_id: CachedComputePipelineId
}

impl FromWorld for GrassComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mesh_layout = render_device.create_bind_group_layout(
            "grass_compute_mesh_layout", 
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                storage_buffer_read_only::<Vec<[f32; 4]>>(false),
            )
        );

        let chunk_layout = render_device.create_bind_group_layout(
            "grass_compute_indices_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<GrassInstanceData>>(false)
                )
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compute_grass.wgsl");

        let compute_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("grass_gen_compute_pipeline".into()),
                layout: vec![mesh_layout.clone(), chunk_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: "main".into()
            });
        
        Self {
            mesh_layout,
            chunk_layout,
            compute_id,
        }
    }
}

#[derive(Resource)]
pub(crate) struct GrassRenderPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
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