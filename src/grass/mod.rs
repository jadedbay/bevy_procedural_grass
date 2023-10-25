use bevy::{prelude::*, render::{extract_component::ExtractComponentPlugin, RenderApp, render_resource::SpecializedMeshPipelines, Render, RenderSet, render_phase::AddRenderCommand}, core_pipeline::core_3d::Opaque3d};

pub mod grass;
pub mod pipeline;
pub mod draw;
pub mod prepare;
pub mod queue;
pub mod extract;

use self::{grass::Grass, draw::DrawGrass, pipeline::GrassPipeline, extract::{GrassColorData, GrassInstanceData}};

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .register_type::<GrassColorData>()
        .add_systems(PostStartup, grass::load_grass)
        .add_systems(Update, (grass::update_grass, grass::update_grass_color))
        .add_plugins(ExtractComponentPlugin::<GrassInstanceData>::default())
        .add_plugins(ExtractComponentPlugin::<GrassColorData>::default())
        .sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
        .add_systems(
            Render,
            (
                queue::grass_queue.in_set(RenderSet::Queue),
                prepare::prepare_instance_buffers.in_set(RenderSet::Prepare),
                prepare::prepare_color_buffers.in_set(RenderSet::Prepare),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<GrassPipeline>();
    }
}