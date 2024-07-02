use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, primitives::Aabb, render_graph::RenderGraphApp, render_phase::AddRenderCommand, Render, RenderApp, RenderSet}};
use grass::chunk::create_chunks;
use pipeline::GrassComputePipeline;

mod pipeline;
mod prepare;
mod node;
pub mod grass;
pub mod util;

pub mod prelude {
    pub use crate::ProceduralGrassPlugin;
    pub use crate::grass::{Grass, GrassBundle, chunk::GrassChunk};
}

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, create_chunks);

        // let render_app = app.sub_app_mut(RenderApp);

        // render_app
        //     .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<GrassComputePipeline>();
    }
}