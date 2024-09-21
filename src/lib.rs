use bevy::{asset::embedded_asset, core_pipeline::core_3d::{graph::Core3d, Opaque3d}, pbr::graph::NodePbr, prelude::*, render::{extract_component::ExtractComponentPlugin, graph::CameraDriverLabel, render_graph::{RenderGraph, RenderGraphApp}, render_phase::AddRenderCommand, render_resource::{SpecializedComputePipelines, SpecializedMeshPipelines}, Render, RenderApp, RenderSet}};

use grass::{chunk::create_chunks, Grass};
use precompute::pipeline::{GrassPrecomputePipeline, GrassComputeChunkPipeline};
use prefix_sum::PrefixSumPipeline;
use render::{node::{compute_grass, ComputeGrassNode, ComputeGrassNodeLabel}, pipeline::GrassComputePipeline};

use crate::{grass::ground_mesh::{prepare_ground_mesh, GroundMesh}, render::{draw::DrawGrass, node::{CullGrassNode, CullGrassNodeLabel}, pipeline::GrassRenderPipeline, prepare::prepare_grass, queue::queue_grass}};

mod precompute;
mod render;
mod prefix_sum;
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
        embedded_asset!(app, "shaders/dispatch_counts.wgsl");
        embedded_asset!(app, "shaders/grass_cull.wgsl");

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
                    prepare_grass.in_set(RenderSet::PrepareBindGroups),
                    compute_grass.after(RenderSet::PrepareBindGroups).before(RenderSet::Render), // dont know if .after is required?
                    // prepare_ground_mesh_bindgroup.in_set(RenderSet::PrepareBindGroups),
                )   
            );
        // render_app.add_render_graph_node::<ComputeGrassNode>(Core3d, ComputeGrassNodeLabel);
        render_app.add_render_graph_node::<CullGrassNode>(Core3d, CullGrassNodeLabel);

        render_app.add_render_graph_edges(
        Core3d, 
        (
            NodePbr::ShadowPass, 
            // ComputeGrassNodeLabel,
            CullGrassNodeLabel,
        )
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<PrefixSumPipeline>()
            .init_resource::<GrassComputeChunkPipeline>()
            .init_resource::<GrassComputePipeline>()
            .init_resource::<GrassPrecomputePipeline>()
            .init_resource::<GrassRenderPipeline>();
    }
}
