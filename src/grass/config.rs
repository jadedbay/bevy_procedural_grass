use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GrassConfig {
    pub cull_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 200.0,
        }
    }
}
