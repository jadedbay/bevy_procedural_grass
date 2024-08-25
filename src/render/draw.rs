use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup}, render::{mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}}};

use super::prepare::GrassBufferBindGroup;

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
    type ItemQuery = Read<GrassBufferBindGroup>;

    #[inline]
    fn render<'w>(
            item: &P,
            _view: ROQueryItem<'w, Self::ViewQuery>,
            grass_bind_groups: Option<ROQueryItem<'w, Self::ItemQuery>>,
            (meshes, render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
            pass: &mut TrackedRenderPass<'w>,
        ) -> RenderCommandResult {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity()) else { 
                return RenderCommandResult::Failure; 
            };

            let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
                return RenderCommandResult::Failure;
            };

            let Some(grass_bind_groups) = grass_bind_groups else {
                return RenderCommandResult::Failure;
            };

            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count: _,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    for chunk in &grass_bind_groups.chunks {
                        pass.set_vertex_buffer(1, chunk.compact_buffer.slice(..));
                        pass.draw_indexed_indirect(&chunk.indirect_args_buffer, 0);
                    }
                }
                GpuBufferInfo::NonIndexed => {} // will always be indexed
            }
            
            RenderCommandResult::Success
    }
}