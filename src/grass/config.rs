use bevy::{prelude::*, render::extract_resource::ExtractResource};

use super::cull::GrassCullChunks;

#[derive(Resource, Clone, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct GrassConfig {
    pub cull_distance: f32,
    pub gpu_culling: bool,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 250.0,
            gpu_culling: true,
        }
    }
}

pub(crate) fn reload_grass_chunks(
    mut commands: Commands,
    grass_config: Res<GrassConfig>,
    mut gpu_culling: Local<bool>,
    mut query: Query<(Entity, &mut GrassCullChunks)>,
) {
    if grass_config.gpu_culling != *gpu_culling {
        *gpu_culling = grass_config.gpu_culling;

        for (entity, mut cull_chunks) in &mut query {
            for (_, chunk_entity) in cull_chunks.0.iter() {
                commands.entity(entity).remove_children(&[*chunk_entity]);
                commands.entity(*chunk_entity).despawn();
            }
            cull_chunks.0.clear();
        }
    }
}