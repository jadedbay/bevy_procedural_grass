use bevy::{prelude::*, render::{render_asset::RenderAssetPlugin, extract_component::ExtractComponentPlugin, RenderApp, render_resource::SpecializedMeshPipelines, Render, render_phase::AddRenderCommand, RenderSet, extract_resource::ExtractResourcePlugin}, core_pipeline::core_3d::Opaque3d, asset::load_internal_asset};

use grass::{chunk::GrassChunks, grass::{Grass, GrassLODMesh}, wind::GrassWind, config::GrassConfig};
use render::{instance::GrassChunkData, pipeline::GrassPipeline, draw::DrawGrass};

use crate::grass::displacement::GrassTimer;
#[cfg(feature = "bevy-inspector-egui")]
use crate::grass::displacement::GrassDisplacer;

pub mod grass;
mod render;
mod util;

pub mod prelude {
    pub use crate::ProceduralGrassPlugin;
    pub use crate::grass::{
        grass::{GrassBundle, Grass, GrassLODMesh}, 
        mesh::GrassMesh, 
        wind::{GrassWind, Wind},
        displacement::GrassDisplacer,
        config::GrassConfig,
    };
}

pub(crate) const GRASS_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(195_094_223_228_228_028_086_047_086_167_255_040_126);

#[derive(Default, Clone)]
pub struct ProceduralGrassPlugin {
    pub config: GrassConfig,
    pub wind: GrassWind,
}

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            GRASS_SHADER_HANDLE,
            "assets/shaders/grass.wgsl",
            Shader::from_wgsl
        );

        #[cfg(feature = "bevy-inspector-egui")]
        {
            app 
                .register_type::<Grass>()
                .register_type::<GrassWind>()
                .register_type::<GrassDisplacer>()
                .register_type::<GrassConfig>();
        }
        app
            .insert_resource(self.wind.clone())
            .insert_resource(self.config)
            .insert_resource(GrassTimer::default())
            .add_systems(Startup, grass::wind::create_wind_map)
            .add_systems(PostStartup, grass::grass::generate_grass)
            .add_systems(Update, (grass::chunk::grass_culling, grass::displacement::grass_displacement))
            .init_asset::<GrassChunkData>()
            .add_plugins(RenderAssetPlugin::<GrassChunkData>::default())
            .add_plugins((
                ExtractComponentPlugin::<Grass>::default(),
                ExtractComponentPlugin::<GrassChunks>::default(),
                ExtractComponentPlugin::<GrassLODMesh>::default(),
                ExtractComponentPlugin::<GrassWind>::default(),
                ExtractResourcePlugin::<GrassWind>::default(),
            ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_render_command::<Opaque3d, DrawGrass>()
        .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
        .add_systems(
            Render,
            (
                render::queue::grass_queue.in_set(RenderSet::QueueMeshes),
                render::prepare::prepare_grass_buffers.in_set(RenderSet::PrepareResources),
                render::prepare::prepare_global_wind_buffers.in_set(RenderSet::PrepareResources),
                render::prepare::prepare_local_wind_buffers.in_set(RenderSet::PrepareResources),
                render::prepare::prepare_grass_bind_group.in_set(RenderSet::PrepareBindGroups),
                render::prepare::prepare_global_wind_bind_group.in_set(RenderSet::PrepareBindGroups),
                render::prepare::prepare_local_wind_bind_group.in_set(RenderSet::PrepareBindGroups),
                render::prepare::prepare_displacement_bind_group.in_set(RenderSet::PrepareBindGroups),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassPipeline>();
    }
}