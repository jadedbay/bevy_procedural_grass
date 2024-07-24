use bevy::{prelude::*, utils::HashMap};

use super::prepare::GrassBufferBindGroup;

#[derive(Resource, Default)]
pub struct GrassGpuScene {
    pub entities: HashMap<Entity, GrassBufferBindGroup>,
}

#[derive(Component)]
pub struct GrassGpuSceneMarker; 