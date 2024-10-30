use bevy::prelude::*;

#[derive(Clone, Copy)]
pub enum GrassLOD {
    High,
    Low,
} 

#[derive(Component, Default, Clone)]
pub struct GrassLODMesh(pub Option<Handle<Mesh>>);