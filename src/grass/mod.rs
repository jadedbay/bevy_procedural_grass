use bevy::{prelude::*, render::{extract_component::ExtractComponentPlugin, RenderApp, render_resource::{SpecializedMeshPipelines, Buffer}, Render, RenderSet, render_phase::AddRenderCommand, render_asset::RenderAssetPlugin, Extract, render_graph::RenderGraph}, core_pipeline::core_3d::Opaque3d};

pub mod grass;
pub mod pipeline;
pub mod draw;
pub mod prepare;
pub mod queue;
pub mod extract;
pub mod wind;
pub mod compute_pipeline;
pub mod chunk;

use self::{grass::Grass, draw::DrawGrass, pipeline::GrassPipeline, extract::{GrassColorData, GrassInstanceData, WindData, BladeData}, wind::WindMap, compute_pipeline::{GrassComputePipeline, ComputeNode}, chunk::{GrassChunks, GrassToDraw}};

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .add_systems(PostStartup, grass::load_grass)
        .add_systems(Update, (grass::update_grass_data, grass::update_grass_params, chunk::grass_culling))
        .init_asset::<GrassInstanceData>()
        .add_plugins(RenderAssetPlugin::<GrassInstanceData>::default())
        .add_plugins(ExtractComponentPlugin::<GrassColorData>::default())
        .add_plugins(ExtractComponentPlugin::<WindData>::default())
        .add_plugins(ExtractComponentPlugin::<BladeData>::default())
        .add_plugins(ExtractComponentPlugin::<GrassToDraw>::default())
        .add_plugins(ExtractComponentPlugin::<WindMap>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
        //.add_systems(ExtractSchedule, extract::extract_grass)
        .add_systems(
            Render,
            (
                queue::grass_queue.in_set(RenderSet::QueueMeshes),
                prepare::prepare_color_buffers.in_set(RenderSet::PrepareBindGroups),
                prepare::prepare_wind_buffers.in_set(RenderSet::PrepareBindGroups),
                prepare::prepare_blade_buffers.in_set(RenderSet::PrepareBindGroups),
                prepare::prepare_wind_map_buffers.in_set(RenderSet::PrepareBindGroups),
                compute_pipeline::prepare_compute_bind_group.in_set(RenderSet::PrepareBindGroups),
            ),
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("grass_compute", ComputeNode::default());
        render_graph.add_node_edge("grass_compute", bevy::render::main_graph::node::CAMERA_DRIVER);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassPipeline>()
            .init_resource::<GrassComputePipeline>();
    }
}