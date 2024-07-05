use bevy::{ecs::system::{lifetimeless::SRes, SystemParamItem}, prelude::*, render::{mesh::VertexAttributeValues, render_asset::{PrepareAssetError, RenderAsset, RenderAssetUsages}, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

pub(crate) struct GrassBaseMesh {
    pub(crate) positions_buffer: Buffer,
}


impl RenderAsset for GrassBaseMesh {
    type SourceAsset = Mesh;
    type Param = SRes<RenderDevice>;

    #[inline]
    fn asset_usage(_source_asset: &Self::SourceAsset) -> RenderAssetUsages {
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    }

    fn byte_len(mesh: &Self::SourceAsset) -> Option<usize> {
        Some(mesh.count_vertices() * std::mem::size_of::<Vec4>())
    }

    fn prepare_asset(
            mesh: Self::SourceAsset,
            render_device: &mut SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
            let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(VertexAttributeValues::Float32x3(positions)) => positions,
                _ => {
                    warn!("Mesh does not contain positions");
                    return Err(PrepareAssetError::RetryNextUpdate(mesh));
                },
            };

            let padded_positions: Vec<[f32; 4]> = positions.iter().map(|x| [x[0], x[1], x[2], 0.0]).collect();

            let positions_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("grass_mesh_positions_buffer"),
                contents: bytemuck::cast_slice(padded_positions.as_slice()),
                usage: BufferUsages::STORAGE
            });
 
            Ok(GrassBaseMesh {
                positions_buffer,
            })
    }
}
