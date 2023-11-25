use bevy::{prelude::*, render::{extract_component::ExtractComponent, render_asset::RenderAsset, renderer::RenderDevice, render_resource::{BufferInitDescriptor, BufferUsages}}, ecs::{query::QueryItem, system::lifetimeless::SRes}, reflect::TypeUuid};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

use crate::grass::{grass::{GrassColor, Blade}, wind::Wind};

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

#[derive(Component, Clone, Copy, Pod, Zeroable, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct WindData {
    pub speed: f32,
    pub strength: f32,
    pub direction: f32,
    pub force: f32,
}

impl From<Wind> for WindData {
    fn from(wind: Wind) -> Self {
        Self {
            speed: wind.speed,
            strength: wind.strength,
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

#[derive(Component, Clone, Copy, Pod, Zeroable, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct BladeData {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub bend: f32,
}

impl From<Blade> for BladeData {
    fn from(blade: Blade) -> Self {
        Self {
            length: blade.length,
            width: blade.width,
            tilt: blade.tilt,
            bend: blade.bend,
        }
    }
}

impl ExtractComponent for BladeData {
    type Query = &'static BladeData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}