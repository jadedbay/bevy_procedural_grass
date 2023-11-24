use bevy::{prelude::*, render::{extract_component::ExtractComponent, Extract, render_asset::RenderAsset, renderer::RenderDevice, render_resource::{BufferInitDescriptor, BufferUsages}}, ecs::{query::QueryItem, system::lifetimeless::SRes}, reflect::TypeUuid};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use bytemuck::{Pod, Zeroable};

use super::{grass::{GrassColor, Blade, GrassDataBuffer}, wind::Wind};

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
    pub normal: Vec3,
    pub uv: Vec2,
    pub chunk: Vec3,
}

#[derive(Component, Deref, Clone, Asset, TypeUuid, TypePath)]
#[uuid = "81a29e63-ef6c-4561-b49c-4a138ff39c01"]
pub struct GrassInstanceData(pub Vec<InstanceData>);

impl Default for GrassInstanceData {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl RenderAsset for GrassInstanceData {
    type ExtractedAsset = GrassInstanceData;
    type PreparedAsset = GrassDataBuffer;
    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        dbg!("extract");
        GrassInstanceData(self.0.clone())
    }

    fn prepare_asset(
            extracted_asset: Self::ExtractedAsset,
            param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self::PreparedAsset, bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>> {
        dbg!("prepare");
        let render_device = param;

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents:  bytemuck::cast_slice(extracted_asset.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE
        });

        Ok(GrassDataBuffer {
            buffer,
            length: extracted_asset.len(),
        })
    }
}

// pub fn extract_grass(
//     mut commands: Commands,
//     extract: Extract<Query<(Entity, &GrassHandles)>>
// ) {
//     let mut values = Vec::new();
//     for (entity, data) in extract.iter() {
//         values.push((entity, data.clone()))
//     }
//     commands.insert_or_spawn_batch(values);
// }



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