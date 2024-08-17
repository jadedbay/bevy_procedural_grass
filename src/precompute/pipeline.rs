use bevy::{prelude::*, render::{render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_read_only_sized, storage_buffer_sized, uniform_buffer}, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderDefVal, ShaderStages, SpecializedComputePipeline}, renderer::RenderDevice}};

#[derive(Resource)]
pub struct GrassPrecomputePipeline {
    pub mesh_layout: BindGroupLayout,
    pub triangle_count_pipeline_id: CachedComputePipelineId,
    pub expand_pipeline_id: CachedComputePipelineId, 
}
impl FromWorld for GrassPrecomputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mesh_layout = render_device.create_bind_group_layout(
            "grass_compute_mesh_layout", 
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<[f32; 4]>>(false),
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                    uniform_buffer::<f32>(false),
                )
            )
        );

        let expand_layout = render_device.create_bind_group_layout(
            "expand_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer_sized(false, None),
                )
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/area.wgsl");
        let expand_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/expand.wgsl");

        let triangle_count_pipeline_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("ground_compute_area_pipeline".into()),
                layout: vec![mesh_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: "main".into(),
            });

        let expand_pipeline_id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("expand_pipeline".into()),
                layout: vec![expand_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: expand_shader,
                shader_defs: vec![],
                entry_point: "main".into(),
            });

        Self {
            mesh_layout,
            triangle_count_pipeline_id,
            expand_pipeline_id,
        }
    }
}


#[derive(Component)]
pub struct GrassComputeChunkPipelineId(pub CachedComputePipelineId);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GrassComputeChunkPipelineKey {
    pub workgroup_count: UVec3,
}

#[derive(Resource)]
pub struct GrassComputeChunkPipeline {
    pub mesh_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl SpecializedComputePipeline for GrassComputeChunkPipeline {
    type Key = GrassComputeChunkPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let shader_defs = vec![
            ShaderDefVal::UInt("CHUNK_SIZE_X".into(), key.workgroup_count.x),
            ShaderDefVal::UInt("CHUNK_SIZE_Y".into(), key.workgroup_count.y),
            ShaderDefVal::UInt("CHUNK_SIZE_Z".into(), key.workgroup_count.z),
        ];

        ComputePipelineDescriptor {
            label: Some("compute_chunk_pipeline".into()),
            layout: vec![self.mesh_layout.clone()],
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }],
            shader: self.shader.clone(),
            shader_defs,
            entry_point: "main".into(),
        }
    } 
}

impl FromWorld for GrassComputeChunkPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let mesh_layout = render_device.create_bind_group_layout(
            "grass_compute_mesh_layout", 
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<[f32; 4]>>(false),
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                )
            )
        );

        let shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compute_chunk.wgsl");
        
        Self {
            mesh_layout,
            shader,
        }
    }
}
