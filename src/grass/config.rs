use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::{RenderDevice, RenderQueue}}};

use super::cull::GrassCullChunks;

#[derive(Resource, Clone, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct GrassConfig {
    pub cull_distance: f32,
    pub grass_shadows: bool,
    pub shadow_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 250.0,
            grass_shadows: true,
            shadow_distance: 20.0,
        }
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct GrassConfigGpu {
    pub shadow_distance_buffer: Buffer,
}

pub(crate) fn init_config_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    config: Res<GrassConfig>,
) {
    commands.insert_resource(
        GrassConfigGpu {
            shadow_distance_buffer: render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("shadow_distance_buffer"),
                    contents: bytemuck::cast_slice(&[config.shadow_distance]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }
            )
        }
    );
}

pub(crate) fn reload_grass_chunks(
    config: Res<GrassConfig>,
    mut grass_shadows: Local<bool>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GrassCullChunks)>,
) {
    if config.grass_shadows != *grass_shadows {
        *grass_shadows = config.grass_shadows;
        for (entity, mut cull_chunks) in &mut query {
            for (_, chunk_entity) in cull_chunks.0.iter() {
                commands.entity(entity).remove_children(&[*chunk_entity]);
                commands.entity(*chunk_entity).despawn();
            }
            cull_chunks.0.clear();
        }
    }
}

pub(crate) fn update_config_buffers(
    render_queue: Res<RenderQueue>,
    config: Res<GrassConfig>,
    config_buffers: Res<GrassConfigGpu>,
    mut shadow_distance: Local<f32>,
) {
    if config.shadow_distance != *shadow_distance {
        render_queue.0.write_buffer(
            &config_buffers.shadow_distance_buffer, 
            0, 
            bytemuck::cast_slice(&[config.shadow_distance]),
        );
        *shadow_distance = config.shadow_distance;
    }
}


