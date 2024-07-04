use bevy::{prelude::*, render::{render_resource::{binding_types::storage_buffer_read_only_sized, BindGroupLayout, BindGroupLayoutEntries, BindingType, BufferBindingType, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages}, renderer::RenderDevice}};

#[derive(Resource)]
pub(crate) struct GrassComputePipeline {
    pub mesh_layout: BindGroupLayout,
    pub compute_id: CachedComputePipelineId
}

impl FromWorld for GrassComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mesh_layout = render_device.create_bind_group_layout(
            "grass_compute_mesh_layout", 
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None), // positions
                    storage_buffer_read_only_sized(false, None), // normals
                )
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_compute.wgsl");

        let compute_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("grass_gen_compute_pipeline".into()),
                layout: vec![mesh_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: "compute".into()
            });
        
        Self {
            mesh_layout,
            compute_id
        }
    }
}