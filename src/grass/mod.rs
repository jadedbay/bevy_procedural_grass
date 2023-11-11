use bevy::{prelude::*, render::{extract_component::ExtractComponentPlugin, RenderApp, render_resource::{SpecializedMeshPipelines, Buffer}, Render, RenderSet, render_phase::AddRenderCommand, render_asset::RenderAssetPlugin, Extract}, core_pipeline::core_3d::Opaque3d};

pub mod grass;
pub mod pipeline;
pub mod draw;
pub mod prepare;
pub mod queue;
pub mod extract;
pub mod wind;

use self::{grass::Grass, draw::DrawGrass, pipeline::GrassPipeline, extract::{GrassColorData, GrassInstanceData, WindData, LightData, BladeData}, wind::WindMap};

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .add_systems(PostStartup, grass::load_grass)
        .add_systems(Update, (grass::update_grass_data, grass::update_grass_params, grass::update_light))
        .add_asset::<GrassInstanceData>()
        .add_plugins(RenderAssetPlugin::<GrassInstanceData>::default())
        .add_plugins(ExtractComponentPlugin::<GrassColorData>::default())
        .add_plugins(ExtractComponentPlugin::<WindData>::default())
        .add_plugins(ExtractComponentPlugin::<LightData>::default())
        .add_plugins(ExtractComponentPlugin::<BladeData>::default())
        .add_plugins(ExtractComponentPlugin::<WindMap>::default())
        .sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
        .add_systems(ExtractSchedule, extract::extract_grass)
        .add_systems(
            Render,
            (
                queue::grass_queue.in_set(RenderSet::Queue),
                prepare::prepare_color_buffers.in_set(RenderSet::Prepare),
                prepare::prepare_wind_buffers.in_set(RenderSet::Prepare),
                prepare::prepare_light_buffers.in_set(RenderSet::Prepare),
                prepare::prepare_blade_buffers.in_set(RenderSet::Prepare),
                prepare::prepare_wind_map_buffers.in_set(RenderSet::Prepare),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<GrassPipeline>();
    }
}