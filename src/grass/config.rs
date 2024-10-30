use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::{Buffer, BufferInitDescriptor, BufferUsages, ShaderType}, renderer::{RenderDevice, RenderQueue}}};

use super::{chunk::unload_chunks, cull::GrassCullChunks};

#[derive(Resource, Clone, ExtractResource, Reflect)]
#[reflect(Resource)]
pub struct GrassConfig {
    pub cull_distance: f32,
    pub lod_distance: f32,
    pub grass_shadows: GrassCastShadows,
    pub shadow_distance: f32,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            cull_distance: 250.0,
            lod_distance: 50.0,
            grass_shadows: GrassCastShadows::default(),
            shadow_distance: 20.0,
        }
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub enum GrassCastShadows {
    Enabled(GrassLightTypes),
    Disabled,
}
impl Default for GrassCastShadows {
    fn default() -> Self {
        Self::Enabled(GrassLightTypes {
            directional: true,
            point: false,
            spot: false,
        })
    }
}
impl GrassCastShadows {
    pub fn enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }

    pub fn light_enabled(&self, light_type: GrassLightType) -> bool {
        match self {
            Self::Enabled(types) => types.is_enabled(light_type),
            Self::Disabled => false,
        }
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct GrassLightTypes {
    directional: bool,
    point: bool,
    spot: bool,
}
impl Default for GrassLightTypes {
    fn default() -> Self {
        Self {
            directional: true,
            point: false,
            spot: false,
        }
    }
}

impl GrassLightTypes {
    pub fn is_enabled(&self, light_type: GrassLightType) -> bool {
        match light_type {
            GrassLightType::Directional => self.directional,
            GrassLightType::Point => self.point,
            GrassLightType::Spot => self.spot,
        }
    }
}

#[derive(Clone, Copy)]
pub enum GrassLightType {
    Directional,
    Point,
    Spot,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub struct GrassConfigGpu {
    pub shadow_distance: f32,
    pub lod_distance: f32,
}
impl From<GrassConfig> for GrassConfigGpu {
    fn from(value: GrassConfig) -> Self {
        Self {
            shadow_distance: value.shadow_distance,
            lod_distance: value.lod_distance,
        }
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct GrassConfigBuffer(pub Buffer);

pub(crate) fn init_config_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    config: Res<GrassConfig>,
) {
    commands.insert_resource(
        GrassConfigBuffer(
            render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("shadow_distance_buffer"),
                    contents: bytemuck::cast_slice(&[GrassConfigGpu::from(config.clone())]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }
            )
        )
    );
}

pub(crate) fn toggle_shadows(
    config: Res<GrassConfig>,
    mut shadows_enabled: Local<bool>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GrassCullChunks)>,
) {
    let config_shadows_enabled = config.grass_shadows.enabled();

    if config_shadows_enabled != *shadows_enabled {
        *shadows_enabled = config_shadows_enabled;
        for (entity, mut cull_chunks) in &mut query {
            unload_chunks(&mut commands, entity, &mut cull_chunks); 
        }
    }
}

pub(crate) fn update_config_buffers(
    render_queue: Res<RenderQueue>,
    config: Res<GrassConfig>,
    config_buffers: Res<GrassConfigBuffer>,
    mut shadow_distance: Local<f32>,
    mut lod_distance: Local<f32>,
) {
    if config.shadow_distance != *shadow_distance || config.lod_distance != *lod_distance {
        render_queue.0.write_buffer(
            &config_buffers.0, 
            0, 
            bytemuck::cast_slice(&[GrassConfigGpu::from(config.clone())]),
        );
        *shadow_distance = config.shadow_distance;
    }
}

