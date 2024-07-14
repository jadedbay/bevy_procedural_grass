use bevy::{asset::embedded_asset, core_pipeline::core_3d::Opaque3d, prelude::*, render::{extract_component::ExtractComponentPlugin, graph::CameraDriverLabel, render_asset::RenderAssetPlugin, render_graph::RenderGraph, render_phase::AddRenderCommand, render_resource::SpecializedMeshPipelines, Render, RenderApp, RenderSet}};

use grass::{chunk::create_chunks, Grass, GrassGround};
use render::{compute_mesh::GrassGroundMesh, node::{ComputeGrassNode, ComputeGrassNodeLabel}, pipeline::GrassComputePipeline, prepare::prepare_compute_bind_groups};

use crate::render::{draw::DrawGrass, pipeline::GrassRenderPipeline, prepare::prepare_grass_instance_buffers, queue::queue_grass};

mod render;
pub mod grass;
pub mod util;

pub mod prelude {
    pub use crate::ProceduralGrassPlugin;
    pub use crate::grass::{Grass, GrassBundle, GrassGround, chunk::GrassChunk, mesh::GrassMesh};
}

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/compute_grass.wgsl");
        embedded_asset!(app, "shaders/grass.wgsl");

        app
            .add_plugins((
                RenderAssetPlugin::<GrassGroundMesh>::default(),
                ExtractComponentPlugin::<Grass>::default(),
                ExtractComponentPlugin::<GrassGround>::default(),
            ))
            .add_systems(PostStartup, create_chunks);

        let render_app = app.sub_app_mut(RenderApp);
        let compute_node = ComputeGrassNode::from_world(render_app.world_mut());

        render_app
            .add_render_command::<Opaque3d, DrawGrass>()
            .init_resource::<SpecializedMeshPipelines<GrassRenderPipeline>>()
            .add_systems(
                Render, 
                (
                    queue_grass.in_set(RenderSet::QueueMeshes),
                    prepare_grass_instance_buffers.in_set(RenderSet::PrepareResources),
                    prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups)
                )   
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(ComputeGrassNodeLabel, compute_node);
        render_graph.add_node_edge(ComputeGrassNodeLabel, CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassComputePipeline>()
            .init_resource::<GrassRenderPipeline>();
    }
}