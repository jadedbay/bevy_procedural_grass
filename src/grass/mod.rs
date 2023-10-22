use bevy::prelude::*;

pub mod grass;
pub mod pipeline;

use self::grass::{Grass, GrassColorData};

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grass>()
        .register_type::<GrassColorData>()
        .add_systems(PostStartup, grass::load_grass)
        .add_systems(Update, (grass::update_grass, grass::update_grass_color));
    }
}