use bevy::{prelude::*, render::extract_component::ExtractComponent, ecs::query::QueryItem};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

use super::grass::GrassColor;

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

#[derive(Clone, Copy, Pod, Zeroable, Reflect)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
}

#[derive(Component, Deref, Clone, Reflect)]
pub struct GrassInstanceData(pub Vec<InstanceData>);

impl Default for GrassInstanceData {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl ExtractComponent for GrassInstanceData {
    type Query = &'static GrassInstanceData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(GrassInstanceData(item.0.clone()))
    }
}