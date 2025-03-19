use std::marker::PhantomData;

use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{RenderMeshInstances, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup, SetPrepassViewBindGroup}, prelude::Component, render::{mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}, render_resource::Buffer}};

use crate::{grass::lod::GrassLOD, prelude::GrassLODMesh, GrassMaterial};

use super::prepare::{CompactBindGroups, CompactBindGroupsLOD, GrassShadowBindGroups};

pub(crate) type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced<CompactBindGroups>,
);

pub(crate) type DrawGrassLOD = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced<CompactBindGroups>,
    DrawGrassLODInstanced,
);

pub(crate) type DrawGrassPrepass = (
    SetItemPipeline,
    SetPrepassViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced<GrassShadowBindGroups>,
);

trait GrassBindGroups: Component {
    fn compact_buffer(&self) -> &Buffer;
    fn indirect_args_buffer(&self) -> &Buffer;
}
impl GrassBindGroups for CompactBindGroups {
    fn compact_buffer(&self) -> &Buffer { &self.compact_buffer }
    fn indirect_args_buffer(&self) -> &Buffer { &self.indirect_args_buffer }
}
impl GrassBindGroups for GrassShadowBindGroups {
    fn compact_buffer(&self) -> &Buffer { &self.0.compact_buffer }
    fn indirect_args_buffer(&self) -> &Buffer { &self.0.indirect_args_buffer }
}

#[allow(private_bounds)]
pub(crate) struct DrawGrassInstanced<B: GrassBindGroups>(PhantomData<B>);
impl<P: PhaseItem, B: GrassBindGroups> RenderCommand<P> for DrawGrassInstanced<B> {
    type Param = (
        SRes<RenderAssets<RenderMesh>>, 
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<B>;
    #[inline]
    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        query_item: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();


        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity()) else { 
            return RenderCommandResult::Skip; 
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };

        let Some(bind_groups) = query_item else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) = mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };

        
        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        
        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count: _,
            } => {
                let Some(index_buffer_slice) = mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id) else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.set_vertex_buffer(1, bind_groups.compact_buffer().slice(..));
                pass.draw_indexed_indirect(&bind_groups.indirect_args_buffer(), 0);
            }
            RenderMeshBufferInfo::NonIndexed => unreachable!()
        }
        
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawGrassLODInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassLODInstanced {
    type Param = (SRes<RenderAssets<RenderMesh>>, SRes<MeshAllocator>);

    type ViewQuery = ();
    type ItemQuery = (Read<CompactBindGroupsLOD>, Read<GrassLODMesh>);
    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        query_item: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (meshes, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some((lod_bind_groups, lod_mesh)) = query_item else {
            return RenderCommandResult::Skip;
        };
        let Some(ref lod_mesh_handle) = lod_mesh.0 else {
            return RenderCommandResult::Skip;
        };
        let Some(lod_mesh) = meshes.into_inner().get(lod_mesh_handle.id()) else {
            return RenderCommandResult::Skip;
        };

        let Some(vertex_buffer_slice) = mesh_allocator.mesh_vertex_slice(&lod_mesh_handle.id()) else {
            return RenderCommandResult::Skip;
        };
         
        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        match &lod_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count: _,
            } => {
                let Some(index_buffer_slice) = mesh_allocator.mesh_index_slice(&lod_mesh_handle.id()) else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.set_vertex_buffer(1, lod_bind_groups.0.compact_buffer.slice(..));
                pass.draw_indexed_indirect(&lod_bind_groups.0.indirect_args_buffer, 0);
            }
            RenderMeshBufferInfo::NonIndexed => unreachable!()
        }

        RenderCommandResult::Success
    }
}
