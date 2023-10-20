use bevy::prelude::*;

pub mod terrain;
pub mod grass;

use self::grass::Grass;

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .add_systems(PostStartup, grass::load_grass)
        .add_systems(Update, grass::update_grass);
    }
}