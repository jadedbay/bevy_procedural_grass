use bevy::{asset::embedded_asset, core_pipeline::core_3d::{graph::Core3d, Opaque3d}, pbr::graph::NodePbr, prelude::*, render::{extract_component::ExtractComponentPlugin, graph::CameraDriverLabel, render_graph::{RenderGraph, RenderGraphApp}, render_phase::AddRenderCommand, render_resource::SpecializedMeshPipelines, Render, RenderApp, RenderSet}};

use grass::{chunk::create_chunks, Grass};
use render::{node::{ComputeGrassNode, ComputeGrassNodeLabel}, pipeline::{GrassComputePPSPipelines, GrassComputePipeline}, prepare::prepare_grass_bind_groups};

use crate::{grass::ground_mesh::{prepare_ground_mesh, GroundMesh}, render::{draw::DrawGrass, pipeline::GrassRenderPipeline, queue::queue_grass}};

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
        embedded_asset!(app, "shaders/grass_types.wgsl");
        embedded_asset!(app, "shaders/compute_grass.wgsl");
        embedded_asset!(app, "shaders/scan.wgsl");
        embedded_asset!(app, "shaders/scan_blocks.wgsl");
        embedded_asset!(app, "shaders/compact.wgsl");
        embedded_asset!(app, "shaders/grass.wgsl");

        app
            .add_plugins((
                ExtractComponentPlugin::<Grass>::default(),
                ExtractComponentPlugin::<GroundMesh>::default(),
            ))
            .add_systems(PostStartup, (create_chunks, prepare_ground_mesh));

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_command::<Opaque3d, DrawGrass>()
            .init_resource::<SpecializedMeshPipelines<GrassRenderPipeline>>()
            .add_systems(
                Render, 
                (
                    queue_grass.in_set(RenderSet::QueueMeshes),
                    prepare_grass_bind_groups.in_set(RenderSet::PrepareBindGroups)
                )   
            );
        // let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        // render_graph.add_node(ComputeGrassNodeLabel, compute_node);
        // render_graph.add_node_edge(ComputeGrassNodeLabel, CameraDriverLabel);
        render_app.add_render_graph_node::<ComputeGrassNode>(Core3d, ComputeGrassNodeLabel);
        render_app.add_render_graph_edges(Core3d, (NodePbr::ShadowPass, ComputeGrassNodeLabel));
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassComputePipeline>()
            .init_resource::<GrassComputePPSPipelines>()
            .init_resource::<GrassRenderPipeline>();
    }
}