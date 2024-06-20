use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, primitives::Aabb, render_graph::RenderGraphApp, render_phase::{AddRenderCommand, RenderPhase}, Render, RenderApp, RenderSet}};
use prepare::prepare_compute_bind_groups;

mod pipeline;
mod prepare;
mod node;
pub mod grass;

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        // let render_app = app.sub_app_mut(RenderApp);

        // render_app
        //     .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

    }
}