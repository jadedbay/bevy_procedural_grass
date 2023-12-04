use bevy::{prelude::*, render::extract_component::ExtractComponent, ecs::query::QueryItem};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

use crate::grass::{grass::{GrassColor, Blade, Grass}, wind::Wind};

impl ExtractComponent for Grass {
    type Query = &'static Grass;
    type Filter = ();
    type Out = (GrassColor, Blade);

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some((item.color.clone(), item.blade.clone()))
    }
}

#[derive(Component, Clone, Copy, Pod, Zeroable, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct WindData {
    pub speed: f32,
    pub strength: f32,
    pub variance: f32,
    pub direction: f32,
    pub force: f32,
}

impl From<Wind> for WindData {
    fn from(wind: Wind) -> Self {
        Self {
            speed: wind.speed,
            strength: wind.strength,
            variance: wind.variance,
            direction: wind.direction,
            force: wind.force,
        }
    }
}

impl ExtractComponent for WindData {
    type Query = &'static WindData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}