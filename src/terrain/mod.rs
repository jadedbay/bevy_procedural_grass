use bevy::prelude::*;

use self::component::Terrain;

pub mod component;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Terrain>()
            .add_systems(Startup, component::generate_mesh_on_startup)
            .add_systems(PostUpdate, component::update_mesh); 
    }
}