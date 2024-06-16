use bevy::{prelude::*, render::primitives::Aabb};

mod pipeline;

pub struct ProceduralGrassPlugin;

impl Plugin for ProceduralGrassPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Bundle, Default)]
pub struct GrassBundle {
    aabb: Aabb
}