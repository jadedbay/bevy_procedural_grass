use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, primitives::Aabb, render_graph::RenderGraphApp, render_phase::AddRenderCommand, Render, RenderApp, RenderSet}};
use grass::create_chunks;
use prepare::prepare_compute_bind_groups;

mod pipeline;
mod prepare;
mod node;
pub mod grass;
pub mod util;

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, create_chunks);

        // let render_app = app.sub_app_mut(RenderApp);

        // render_app
        //     .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

    }
}