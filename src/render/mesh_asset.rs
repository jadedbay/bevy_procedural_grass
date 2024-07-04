use bevy::{ecs::system::lifetimeless::SRes, prelude::*, render::{mesh::VertexAttributeValues, render_asset::{PrepareAssetError, RenderAsset, RenderAssetUsages}, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

pub(crate) struct GrassBaseMesh {
    positions_buffer: Buffer,
}

impl RenderAsset for GrassBaseMesh {
    type SourceAsset = Mesh;
    type Param = SRes<RenderDevice>;

    fn asset_usage(_source_asset: &Self::SourceAsset) -> bevy::render::render_asset::RenderAssetUsages {
        RenderAssetUsages::RENDER_WORLD
    }

    fn prepare_asset(
            mesh: Self::SourceAsset,
            render_device: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>> {
            let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(VertexAttributeValues::Float32x3(positions)) => positions,
                _ => {
                    warn!("Mesh does not contain positions");
                    return Err(PrepareAssetError::RetryNextUpdate(mesh));
                },
            };

            let positions_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("grass_mesh_positions_buffer"),
                contents: bytemuck::cast_slice(positions.as_slice()),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
            });
            
            Ok(GrassBaseMesh {
                positions_buffer,
            })
    }
}