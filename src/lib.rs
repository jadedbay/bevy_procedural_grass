use bevy::{prelude::*, render::{render_asset::RenderAssetPlugin, extract_component::ExtractComponentPlugin, RenderApp, render_resource::SpecializedMeshPipelines, Render, render_phase::AddRenderCommand, RenderSet, extract_resource::ExtractResourcePlugin}, core_pipeline::core_3d::Opaque3d};

use grass::{chunk::GrassChunks, grass::Grass, wind::{WindMap, GrassWind}};
use render::{instance::GrassInstanceData, pipeline::GrassPipeline, draw::DrawGrass};

pub mod grass;
pub mod render;

#[derive(Default)]
pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .register_type::<GrassWind>()
        .init_resource::<GrassWind>()
        .add_systems(Startup, grass::wind::create_wind_map)
        .add_systems(PostStartup, grass::grass::generate_grass)
        .add_systems(Update, grass::chunk::grass_culling)
        .init_asset::<GrassInstanceData>()
        .add_plugins(RenderAssetPlugin::<GrassInstanceData>::default())
        .add_plugins(ExtractComponentPlugin::<Grass>::default())
        .add_plugins(ExtractComponentPlugin::<GrassChunks>::default())
        .add_plugins(ExtractComponentPlugin::<WindMap>::default())
        .add_plugins(ExtractResourcePlugin::<GrassWind>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
        .add_systems(
            Render,
            (
                render::queue::grass_queue.in_set(RenderSet::QueueMeshes),
                render::prepare::prepare_grass_buffers.in_set(RenderSet::PrepareBindGroups),
                render::prepare::prepare_wind_buffers.in_set(RenderSet::PrepareBindGroups),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassPipeline>();
    }
}