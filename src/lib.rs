use bevy::{asset::embedded_asset, prelude::*, render::{extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin, render_graph::RenderGraph, Render, RenderApp, RenderSet}};

use grass::{chunk::create_chunks, Grass};
use render::{mesh_asset::GrassBaseMesh, node::{ComputeGrassNode, ComputeGrassNodeLabel}, pipeline::GrassComputePipeline, prepare::prepare_compute_bind_groups};

mod render;
pub mod grass;
pub mod util;

pub mod prelude {
    pub use crate::ProceduralGrassPlugin;
    pub use crate::grass::{Grass, GrassBundle, chunk::GrassChunk};
}

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/compute_grass.wgsl");

        app
            .add_plugins((
                RenderAssetPlugin::<GrassBaseMesh>::default(),
                ExtractComponentPlugin::<Grass>::default(),
            ))
            .add_systems(PostStartup, create_chunks);

        let render_app = app.sub_app_mut(RenderApp);
        let compute_node = ComputeGrassNode::from_world(render_app.world_mut());

        render_app
            .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeGrassNodeLabel, compute_node);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassComputePipeline>();
    }
}