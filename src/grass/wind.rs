use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{prelude::*, render::extract_component::ExtractComponent, ecs::query::QueryItem};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};
use noise::Perlin;

#[derive(Reflect, InspectorOptions, Clone, Copy, Pod, Zeroable)]
#[reflect(InspectorOptions)]
#[repr(C)]
pub struct Wind {
    pub speed: f32,
    pub strength: f32,
    pub variance: f32,
    pub direction: f32,
    pub force: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            speed: 0.15,
            strength: 2.,
            variance: 4.,
            direction: 0.0,
            force: 2.,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct WindMap {
    pub wind_map: Handle<Image>,
}

impl ExtractComponent for WindMap {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(WindMap {
            wind_map: item.wind_map.clone_weak(),
        })
    }
}

#[derive(Resource, Default, Clone, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
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
    pub fn generate_wind_map(size: usize) -> Image {
        let width = size;
        let height = size;
        let perlin = Perlin::new(4);
    
        let mut data = Vec::with_capacity(width * height * 4);
    
        for y in 0..height {
            for x in 0..width {
                let angle_x = 2.0 * PI * (x as f64 / width as f64);
                let angle_y = 2.0 * PI * (y as f64 / height as f64);
                let noise = perlin.get([angle_x.cos(), angle_y.sin()]);
                let color = ((noise + 1.0) / 2.0 * 255.0) as u8;
                data.push(color); 
                data.push(color); 
                data.push(color); 
                data.push(255); 
            }
        }
    
        Image::new(
            Extent3d {
                width: width as u32, 
                height: height as u32, 
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
    wind.wind_map = asset_server.add(GrassWind::generate_wind_map(512));
}