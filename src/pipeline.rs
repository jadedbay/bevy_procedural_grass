use bevy::{prelude::*, render::{render_resource::{BindGroupLayout, BindGroupLayoutEntries, BindingType, BufferBindingType, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages}, renderer::RenderDevice}};

#[derive(Resource)]
pub(crate) struct GrassPipeline {
    layout: BindGroupLayout,
    compute_id: CachedComputePipelineId
}

impl FromWorld for GrassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "grass_layout", 
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/generation.wgsl");

        let compute_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("grass_gen_compute_pipeline".into()),
                layout: vec![layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: "compute".into()
            });
        
        Self {
            layout,
            compute_id
        }
    }
}