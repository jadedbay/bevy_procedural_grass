use bevy::prelude::*;

pub mod terrain;
pub mod grass;

use terrain::Terrain;

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Terrain>()
        .add_systems(PostStartup, (grass::generate_grass_data, grass::spawn_grass))
        .add_systems(Update, grass::spread_grass_blades);
    }
}