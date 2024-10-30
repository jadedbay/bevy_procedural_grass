use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{RenderMeshInstances, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup, SetPrepassViewBindGroup}, prelude::Component, render::{mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}, render_resource::Buffer}};

use crate::{grass::lod::GrassLOD, prelude::GrassLODMesh, GrassMaterial};

use super::prepare::{CompactBindGroups, CompactBindGroupsLOD, GrassShadowBindGroups};

pub(crate) type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced,
);

pub(crate) type DrawGrassLOD = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced,
    DrawGrassLODInstanced,
);

pub(crate) type DrawGrassPrepass = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassPrepassInstanced,
);

pub(crate) struct DrawGrassInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<CompactBindGroups>;
    #[inline]
    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        query_item: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (meshes, render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.entity()) else { 
            return RenderCommandResult::Failure; 
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Failure;
        };
        let Some(bind_groups) = query_item else {
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
                pass.set_vertex_buffer(1, bind_groups.compact_buffer.slice(..));
                pass.draw_indexed_indirect(&bind_groups.indirect_args_buffer, 0);
            }
            GpuBufferInfo::NonIndexed => unreachable!()
        }
        
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawGrassLODInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassLODInstanced {
    type Param = SRes<RenderAssets<GpuMesh>>;

    type ViewQuery = ();
    type ItemQuery = (Read<CompactBindGroupsLOD>, Read<GrassLODMesh>);
    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        query_item: Option<ROQueryItem<'w, Self::ItemQuery>>,
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((lod_bind_groups, lod_mesh)) = query_item else {
            return RenderCommandResult::Failure;
        };
        let Some(ref lod_mesh_handle) = lod_mesh.0 else {
            return RenderCommandResult::Success;
        };
        let Some(lod_mesh) = meshes.into_inner().get(lod_mesh_handle.id()) else {
            return RenderCommandResult::Success;
        };
         
        pass.set_vertex_buffer(0, lod_mesh.vertex_buffer.slice(..));
        match &lod_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count: _,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.set_vertex_buffer(1, lod_bind_groups.0.compact_buffer.slice(..));
                pass.draw_indexed_indirect(&lod_bind_groups.0.indirect_args_buffer, 0);
            }
            GpuBufferInfo::NonIndexed => unreachable!()
        }
        
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawGrassPrepassInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassPrepassInstanced {
    type Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<GrassShadowBindGroups>;
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

        let Some(bind_groups) = grass_bind_groups else {
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
                pass.set_vertex_buffer(1, bind_groups.0.compact_buffer.slice(..));
                pass.draw_indexed_indirect(&bind_groups.0.indirect_args_buffer, 0);
            }
            GpuBufferInfo::NonIndexed => unreachable!()
        }
        
        RenderCommandResult::Success
    }
}