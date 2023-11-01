use bevy::prelude::*;
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Wind {
    pub frequency: f32,
    pub speed: f32,
    pub strength: f32,
    pub noise: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            frequency: 18.,
            speed: 0.05,
            strength: 1.,
            noise: 0.1,
        }
    }
}