use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{DrawMesh, RenderMeshInstances, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup, SetPrepassViewBindGroup}, render::{mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}}};

use crate::{prelude::GrassConfig, GrassMaterial};

use super::prepare::{GrassChunkBindGroups, GrassChunkCullBindGroups};

pub(crate) type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced,
);

pub(crate) type DrawGrassPrepass = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced,
);

pub(crate) struct DrawGrassInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = (Option<Read<GrassChunkBindGroups>>, Option<Read<GrassChunkCullBindGroups>>);

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
            
            let Some((bind_groups, cull_bind_groups)) = grass_bind_groups else {
                return RenderCommandResult::Failure;
            };

            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    if let Some(cull_bind_groups) = cull_bind_groups {
                        pass.set_vertex_buffer(1, cull_bind_groups.compact_buffer.slice(..));
                        pass.draw_indexed_indirect(&cull_bind_groups.indirect_args_buffer, 0);
                    } else if let Some(bind_groups) = bind_groups {
                        pass.set_vertex_buffer(1, bind_groups.instance_buffer.slice(..));
                        pass.draw_indexed(0..*count, 0, 0..bind_groups.instance_count);
                    } else {
                        return RenderCommandResult::Failure;
                    } 
                }
                GpuBufferInfo::NonIndexed => unreachable!()
            }
            
            RenderCommandResult::Success
    }
}