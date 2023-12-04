use bevy::prelude::*;
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};

#[derive(Resource, Clone, Copy, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct GrassConfig {
    pub cull_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 200.,
        }
    }
}