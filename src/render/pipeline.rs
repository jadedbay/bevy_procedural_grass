use bevy::{pbr::MaterialPipeline, prelude::*, render::{render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_read_only_sized, storage_buffer_sized, texture_2d, uniform_buffer, uniform_buffer_sized}, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages, SpecializedComputePipeline, SpecializedComputePipelines, TextureSampleType}, renderer::RenderDevice, view::ViewUniform}};
use crate::{grass::{chunk::{GrassChunk, GrassChunkBuffers}, config::GrassConfigGpu, Grass}, prelude::{GrassConfig, GrassLODMesh, GrassMaterial}, util::aabb::Aabb2dGpu};

use super::instance::GrassInstanceData;

#[derive(Resource)]
pub(crate) struct GrassComputePipeline {
    pub chunk_layout: BindGroupLayout,
    pub clump_layout: BindGroupLayout,
    pub compact_layout: BindGroupLayout,
    pub reset_args_layout: BindGroupLayout,
    pub compute_id: CachedComputePipelineId,
    pub compact_pipeline_id: CachedComputePipelineId,
    pub reset_args_pipeline_id: CachedComputePipelineId,

    _grass_util_shader: Handle<Shader>,
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
                    uniform_buffer::<Aabb2dGpu>(false), //TODO: dynamic offset?
                    uniform_buffer::<Aabb2dGpu>(false),
                )
            )
        );

        let clump_layout = render_device.create_bind_group_layout(
            "clump_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<Aabb2dGpu>(false),
                    uniform_buffer::<Vec2>(false),
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer_read_only_sized(false, None),
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
        let compact_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/compact.wgsl");
        let reset_args_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/reset_args.wgsl");
        
        let material_pipeline = world.resource::<MaterialPipeline<GrassMaterial>>();
        let material_layout = material_pipeline.material_layout.clone();

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

        let compute_id = pipeline_cache
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("grass_gen_compute_pipeline".into()),
                layout: vec![chunk_layout.clone(), clump_layout.clone(), material_layout],
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
            clump_layout,
            compact_layout,
            reset_args_layout,
            compute_id,
            compact_pipeline_id,
            reset_args_pipeline_id,
            _grass_util_shader: world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_util.wgsl")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GrassCullPipelineKey {
    lod: bool,
    shadows: bool,
}

#[derive(Resource)]
pub struct GrassCullPipeline {
    pub cull_layout: BindGroupLayout,
    pub cull_layout_or: BindGroupLayout,
    pub cull_layout_shadow_lod: BindGroupLayout,
    shader: Handle<Shader>,
}
impl FromWorld for GrassCullPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let cull_layout = render_device.create_bind_group_layout(
            "cull_grass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer::<Vec<u32>>(false),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<GrassConfigGpu>(false),
                )
            )
        );
        
        // layout for if shadows OR lod enabled
        let cull_layout_or = render_device.create_bind_group_layout(
            "cull_grass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer::<Vec<u32>>(false),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<GrassConfigGpu>(false),
                    storage_buffer::<Vec<u32>>(false),
                )
            )
        );

        let cull_layout_shadow_lod = render_device.create_bind_group_layout(
            "cull_grass_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only_sized(false, None),
                    storage_buffer::<Vec<u32>>(false),
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<GrassConfigGpu>(false),
                    storage_buffer::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                )
            )
        );



        Self {
            cull_layout,
            cull_layout_or,
            cull_layout_shadow_lod,
            shader: world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/grass_cull.wgsl"),
        }
    }
} 
impl SpecializedComputePipeline for GrassCullPipeline {
    type Key = GrassCullPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let layout = match (key.shadows, key.lod) {
            (false, false) => self.cull_layout.clone(),
            (true, false) | (false, true) => self.cull_layout_or.clone(),
            (true, true) => self.cull_layout_shadow_lod.clone(),
        };

        let mut shader_defs = Vec::new();
        if key.shadows {
            shader_defs.push("SHADOW".into());
        }
        if key.lod {
            shader_defs.push("LOD".into());
        }

        ComputePipelineDescriptor {
            label: Some("cull_grass_pipeline".into()),
            layout: vec![layout],
            push_constant_ranges: Vec::new(),
            shader: self.shader.clone(),
            shader_defs,
            entry_point: "main".into(),
        }
    }
}

pub(crate) fn prepare_cull_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedComputePipelines<GrassCullPipeline>>,
    cull_pipeline: Res<GrassCullPipeline>,
    query: Query<(Entity, &GrassLODMesh, &GrassChunkBuffers), With<GrassChunk>>,
) {
    for (entity, lod_mesh, buffers) in &query {
        let key = GrassCullPipelineKey {
            lod: lod_mesh.0.is_some(),
            shadows: buffers.shadow_buffers.is_some(),
        };

        let pipeline_id = pipelines.specialize(
            &pipeline_cache, 
            &cull_pipeline, 
            key
        );

        commands.entity(entity).insert(GrassCullPipelineId(pipeline_id));
    }
}

#[derive(Component)]
pub struct GrassCullPipelineId(pub CachedComputePipelineId);