use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{prelude::*, render::extract_component::ExtractComponent, ecs::query::QueryItem};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use noise::Perlin;

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Wind {
    pub speed: f32,
    pub strength: f32,
    pub direction: f32,
    pub force: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            speed: 0.1,
            strength: 2.,
            direction: 0.0,
            force: 2.,
        }
    }
}

#[derive(Component, Clone)]
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

#[derive(Resource, Default)]
pub struct GrassWind {
    pub wind_data: Wind,
    pub wind_map: Handle<Image>,
}

use noise::NoiseFn;
use std::f64::consts::PI;

impl GrassWind {
    pub fn generate_wind_map() -> Image {
        let width = 512;
        let height = 512;
        let perlin = Perlin::new(0);
    
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


    pub fn save_perlin_noise_image_as_png(width: u32, height: u32, filename: &str) {
        let image = GrassWind::generate_wind_map();
        let mut imgbuf = ImageBuffer::new(width, height);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let index = (x + y * width) as usize * 4;
            *pixel = Rgba([
                image.data[index],
                image.data[index + 1],
                image.data[index + 2],
                image.data[index + 3],
            ]);
        }

        imgbuf.save(filename).unwrap();
    }
}

use image::{ImageBuffer, Rgba};