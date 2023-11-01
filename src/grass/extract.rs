use bevy::{prelude::*, render::{extract_component::ExtractComponent, Extract}, ecs::query::QueryItem, reflect::TypeUuid};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

use super::{grass::GrassColor, wind::Wind};

#[derive(Component, Clone, Copy, Pod, Zeroable, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct GrassColorData {
    ao: [f32; 4],
    color_1: [f32; 4],
    color_2: [f32; 4],
    tip: [f32; 4],
}

impl From<GrassColor> for GrassColorData {
    fn from(color: GrassColor) -> Self {
        Self {
            ao: color.ao.into(),
            color_1: color.color_1.into(),
            color_2: color.color_2.into(),
            tip: color.tip.into(),
        }
    }
}

impl ExtractComponent for GrassColorData {
    type Query = &'static GrassColorData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Reflect, Debug)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub uv: Vec2,
}

#[derive(Component, Deref, Clone, Reflect, TypeUuid)]
#[uuid = "81a29e63-ef6c-4561-b49c-4a138ff39c01"]
pub struct GrassInstanceData(pub Vec<InstanceData>);

impl Default for GrassInstanceData {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub fn extract_grass(
    mut commands: Commands,
    extract: Extract<Query<(Entity, &Handle<GrassInstanceData>)>>
) {
    let mut values = Vec::new();
    for (entity, data) in extract.iter() {
        values.push((entity, data.clone()))
    }
    commands.insert_or_spawn_batch(values);
}

#[derive(Component, Clone, Copy, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct WindData {
    pub frequency: f32,
    pub speed: f32,
    pub noise: f32,
    pub strength: f32,
}

impl From<Wind> for WindData {
    fn from(wind: Wind) -> Self {
        Self {
            frequency: wind.frequency,
            speed: wind.speed,
            noise: wind.noise,
            strength: wind.strength,
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

#[derive(Component, Pod, Zeroable, Clone, Copy, Reflect, Default, Debug)]
#[reflect(Component)]
#[repr(C)]
pub struct LightData {
    pub direction: Vec3,
    _padding: f32,
}

impl ExtractComponent for LightData {
    type Query = &'static LightData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}