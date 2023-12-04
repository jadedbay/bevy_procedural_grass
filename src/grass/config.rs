use bevy::prelude::*;

#[derive(Resource)]
pub struct GrassConfig {
    cull_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 200.,
        }
    }
}