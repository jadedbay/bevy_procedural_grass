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