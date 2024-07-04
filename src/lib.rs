use bevy::{asset::embedded_asset, prelude::*, render::{extract_component::ExtractComponentPlugin, mesh::GpuMesh, render_asset::{ExtractedAssets, PrepareNextFrameAssets, RenderAssetPlugin}, render_graph::RenderGraph, texture::GpuImage, Render, RenderApp, RenderSet}};

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

        render_app
            .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeGrassNodeLabel, ComputeGrassNode);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassComputePipeline>();
    }
}