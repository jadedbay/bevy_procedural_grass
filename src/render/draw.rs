use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup}, prelude::*, render::{mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}}};

use super::prepare::GrassInstanceBuffer;

pub(crate) type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawGrassInstanced,
);

pub(crate) struct DrawGrassInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<GrassInstanceBuffer>;

    #[inline]
    fn render<'w>(
            item: &P,
            _view: ROQueryItem<'w, Self::ViewQuery>,
            instance_buffer: Option<ROQueryItem<'w, Self::ItemQuery>>,
            (meshes, render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
            pass: &mut TrackedRenderPass<'w>,
        ) -> RenderCommandResult {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity()) else { 
                return RenderCommandResult::Failure; 
            };

            let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
                return RenderCommandResult::Failure;
            };

            let Some(instance_buffer) = instance_buffer else {
                return RenderCommandResult::Failure;
            };

            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
                }
                GpuBufferInfo::NonIndexed => {
                    pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
                }
            }
            RenderCommandResult::Success
    }
}