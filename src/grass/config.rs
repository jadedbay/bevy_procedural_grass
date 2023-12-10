use bevy::prelude::*;
#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};

#[derive(Resource, Clone, Copy)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(Resource, InspectorOptions))]
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