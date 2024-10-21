use std::marker::PhantomData;

use bevy::{ecs::{query::ROQueryItem, system::{lifetimeless::{Read, SRes}, SystemParamItem}}, pbr::{RenderMeshInstances, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup, SetPrepassViewBindGroup}, prelude::Component, render::{mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_phase::{PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass}, render_resource::Buffer}};

use crate::GrassMaterial;

use super::prepare::{GrassChunkCullBindGroups, GrassShadowBindGroups};

pub(crate) type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetMaterialBindGroup<GrassMaterial, 2>,
    DrawGrassInstanced<GrassChunkCullBindGroups>,
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

impl GrassBindGroups for GrassChunkCullBindGroups {
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
    type Param = (SRes<RenderAssets<GpuMesh>>, SRes<RenderMeshInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<B>;

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
                pass.set_vertex_buffer(1, bind_groups.compact_buffer().slice(..));
                pass.draw_indexed_indirect(&bind_groups.indirect_args_buffer(), 0);
            }
            GpuBufferInfo::NonIndexed => unreachable!()
        }
        
        RenderCommandResult::Success
    }
}
