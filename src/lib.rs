use bevy::{prelude::*, render::{render_asset::RenderAssetPlugin, RenderApp}};

use grass::chunk::create_chunks;
use render::{mesh_asset::GrassBaseMesh, pipeline::GrassComputePipeline};

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
        app
            .add_plugins(
                RenderAssetPlugin::<GrassBaseMesh>::default()
            )
            .add_systems(PostStartup, create_chunks);

        // let render_app = app.sub_app_mut(RenderApp);

        // render_app
        //     .add_systems(Render, prepare_compute_bind_groups.in_set(RenderSet::PrepareBindGroups));

    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassComputePipeline>();
    }
}