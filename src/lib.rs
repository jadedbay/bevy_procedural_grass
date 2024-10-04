use bevy::{asset::embedded_asset, core_pipeline::core_3d::{graph::{Core3d, Node3d}, Opaque3d}, pbr::{graph::NodePbr, MaterialPipeline, PreparedMaterial}, prelude::*, render::{extract_component::ExtractComponentPlugin, extract_instances::ExtractInstancesPlugin, render_asset::RenderAssetPlugin, render_graph::RenderGraphApp, render_phase::AddRenderCommand, render_resource::SpecializedMeshPipelines, Render, RenderApp, RenderSet}};

use grass::{chunk::{grass_setup, GrassChunk}, config::GrassConfig, cull::cull_chunks, material::GrassMaterial, Grass};
use prefix_sum::PrefixSumPipeline;
use render::{node::{compute_grass, ResetArgsNode, ResetArgsNodeLabel}, pipeline::GrassComputePipeline, prepare::{update_computed_grass, ComputedGrassEntities}};

use crate::render::{draw::DrawGrass, node::{CullGrassNode, CullGrassNodeLabel}, pipeline::GrassRenderPipeline, prepare::prepare_grass, queue::queue_grass};

mod render;
mod prefix_sum;
pub mod grass;
pub mod util;

pub mod prelude {
    pub use crate::ProceduralGrassPlugin;
    pub use crate::grass::{Grass, GrassBundle, GrassHeightMap, mesh::GrassMesh, config::GrassConfig, material::GrassMaterial, material::GrassMaterialExtension};
}

#[derive(Default)]
pub struct ProceduralGrassPlugin {
    pub config: GrassConfig
}

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/grass_types.wgsl");
        embedded_asset!(app, "shaders/compute_grass.wgsl");
        embedded_asset!(app, "shaders/scan.wgsl");
        embedded_asset!(app, "shaders/scan_blocks.wgsl");
        embedded_asset!(app, "shaders/compact.wgsl");
        embedded_asset!(app, "shaders/grass.wgsl");
        embedded_asset!(app, "shaders/grass_cull.wgsl");
        embedded_asset!(app, "shaders/reset_args.wgsl");

        app
            .register_type::<Grass>()
            .register_type::<Image>()
            .insert_resource(self.config.clone())
            .add_plugins((
                GrassMaterialPlugin,
                ExtractComponentPlugin::<Grass>::default(),
                ExtractComponentPlugin::<GrassChunk>::default(),
            ))
            .add_systems(PostStartup, grass_setup)
            .add_systems(Update, cull_chunks);
        
        let render_app = app.sub_app_mut(RenderApp);
        
        render_app
        .add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassRenderPipeline>>()
            .add_systems(
                Render, 
                (
                    update_computed_grass.before(RenderSet::PrepareResources),
                    queue_grass.in_set(RenderSet::QueueMeshes),
                    prepare_grass.in_set(RenderSet::PrepareBindGroups),
                    compute_grass.after(RenderSet::PrepareBindGroups).before(RenderSet::Render), // dont know if .after is required?
                )   
            );

        render_app.add_render_graph_node::<CullGrassNode>(Core3d, CullGrassNodeLabel);
        render_app.add_render_graph_node::<ResetArgsNode>(Core3d, ResetArgsNodeLabel);

        render_app.add_render_graph_edges(
            Core3d, 
            (
                NodePbr::ShadowPass,
                CullGrassNodeLabel,
            )
        );

        render_app.add_render_graph_edge(
            Core3d, Node3d::EndMainPass, ResetArgsNodeLabel,
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<ComputedGrassEntities>()
            .init_resource::<PrefixSumPipeline>()
            .init_resource::<GrassComputePipeline>()
            .init_resource::<GrassRenderPipeline>();
    }
}

struct GrassMaterialPlugin;
impl Plugin for GrassMaterialPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<GrassMaterial>()
            .register_asset_reflect::<GrassMaterial>() 
            .add_plugins((
                ExtractInstancesPlugin::<AssetId<GrassMaterial>>::extract_visible(),
                RenderAssetPlugin::<PreparedMaterial<GrassMaterial>>::default(),
            ));
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<MaterialPipeline<GrassMaterial>>();
        }
    }
}