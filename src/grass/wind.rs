use bevy::{prelude::*, render::extract_component::ExtractComponent, ecs::query::QueryItem};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};

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
