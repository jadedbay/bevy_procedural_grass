use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::prelude::*;
#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
#[repr(C)]
pub struct Wind {
    pub speed: f32,
    pub strength: f32,
    pub variance: f32,
    pub direction: f32,
    pub force: f32,
    pub oscillation: f32,

    pub scale: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            speed: 0.15,
            strength: 1.,
            variance: 1.,
            direction: 0.0,
            force: 1.,
            oscillation: 1.5,

            scale: 100.,
        }
    }
}

#[derive(Resource, Default, Clone)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(Resource, InspectorOptions))]
pub struct GrassWind {
    pub wind_data: Wind,
    pub wind_map: Handle<Image>,
}

impl ExtractResource for GrassWind {
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

use noise::NoiseFn;
use std::f64::consts::PI;

impl GrassWind {
    pub fn generate_wind_map(size: usize, scale: f64) -> Image {
        let perlin = noise::PerlinSurflet::new(0);
    
        let mut data = Vec::with_capacity(size * size * 4);

        let (x1, y1, x2, y2) = (-1.0, -1.0, 1.0, 1.0);
        for y in 0..size {
            for x in 0..size {
                let s = x as f64 / size as f64;
                let t = y as f64 / size as f64;
                let dx = x2 - x1;
                let dy = y2 - y1;

                let nx = x1 + (s * 2.0 * PI).cos() * (dx / (2.0 * PI));
                let ny = y1 + (t * 2.0 * PI).cos() * (dy / (2.0 * PI));
                let nz = x1 + (s * 2.0 * PI).sin() * (dx / (2.0 * PI));
                let nw = y1 + (t * 2.0 * PI).sin() * (dy / (2.0 * PI));

                let noise = perlin.get([nx * scale, ny * scale, nz * scale, nw * scale]);
                let noise_scaled = ((noise + 1.0) / 2.0 * 16777215.0) as u32;

                let r = ((noise_scaled >> 16) & 255) as u8;
                let g = ((noise_scaled >> 8) & 255) as u8;
                let b = (noise_scaled & 255) as u8;

                data.push(r); 
                data.push(g); 
                data.push(b); 
                data.push(255);
            }
        }
    
        Image::new(
            Extent3d {
                width: size as u32, 
                height: size as u32, 
                depth_or_array_layers: 1
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
        )
    }
}

pub fn create_wind_map(
    mut wind: ResMut<GrassWind>,
    asset_server: Res<AssetServer>,
) {
    wind.wind_map = asset_server.add(GrassWind::generate_wind_map(2048, 4.));
}