use bevy::{prelude::*, reflect::TypeUuid, render::{render_asset::{RenderAsset, PrepareAssetError}, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}, ecs::system::{lifetimeless::SRes, SystemParamItem}};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, Reflect, Debug)]
#[repr(C)]
pub struct GrassData {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct GrassDataBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

#[derive(Component, Deref, Clone, Asset, TypeUuid, TypePath)]
#[uuid = "81a29e63-ef6c-4561-b49c-4a138ff39c01"]
pub struct GrassInstanceData(pub Vec<GrassData>);

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
        GrassInstanceData(self.0.clone())
    }

    fn prepare_asset(
            extracted_asset: Self::ExtractedAsset,
            param: &mut SystemParamItem<Self::Param>,
        ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
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