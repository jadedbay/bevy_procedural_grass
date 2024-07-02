use bevy::{prelude::*, render::primitives::Aabb};

pub mod chunk;

use chunk::GrassChunks;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    grass: Grass,
}

#[derive(Component)]
pub struct Grass {
    pub chunk_size: f32,
    pub chunks: GrassChunks,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            chunk_size: 3.0,
            chunks: GrassChunks::new(),
        }
    }
}
