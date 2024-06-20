use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    grass: Grass,
}

#[derive(Component, Default)]
pub struct Grass;
