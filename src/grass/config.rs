use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::{RenderDevice, RenderQueue}}};

#[derive(Resource, Clone, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct GrassConfig {
    pub cull_distance: f32,
    pub shadow_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 250.0,
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


