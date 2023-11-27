use std::marker::PhantomData;

use bevy::{prelude::*, render::{render_phase::{SetItemPipeline, PhaseItem, RenderCommand, TrackedRenderPass, RenderCommandResult}, render_asset::RenderAssets, mesh::GpuBufferInfo}, pbr::{SetMeshViewBindGroup, SetMeshBindGroup, RenderMeshInstances}, ecs::system::{lifetimeless::{SRes, Read}, SystemParamItem}};

use crate::grass::{wind::WindMap, chunk::GrassChunkHandles};

use super::{extract::{GrassColorData, WindData, BladeData}, prepare::BufferBindGroup, instance::GrassInstanceData};

pub type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetBindGroup<2, GrassColorData>,
    SetBindGroup<3, WindData>,
    SetBindGroup<4, BladeData>,
    SetBindGroup<5, WindMap>,
    DrawGrassInstanced,
);

pub struct SetBindGroup<const I: usize, T> {
    _marker: PhantomData<T>,
}

impl<P: PhaseItem, const I: usize,  T> RenderCommand<P> for SetBindGroup<I, T> 
where
    T: 'static + Sync + Send,
{
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<BufferBindGroup<T>>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w BufferBindGroup<T>>,
        _meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(bind_group) = bind_group else {
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(I, &bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawGrassInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderMeshInstances>, SRes<RenderAssets<GrassInstanceData>>);
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<GrassChunkHandles>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        chunks: &'w GrassChunkHandles,
        (meshes, render_mesh_instances, grass_data): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(mesh_instance) = render_mesh_instances.get(&item.entity()) else {
            return RenderCommandResult::Failure;
        };
        let gpu_mesh = match meshes.into_inner().get(mesh_instance.mesh_asset_id) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        let grass_data_inner = grass_data.into_inner();

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

        for handle in &chunks.0 {
            let gpu_grass = match grass_data_inner.get(handle) {
                Some(gpu_grass) => gpu_grass,
                None => return RenderCommandResult::Failure,
            };

            pass.set_vertex_buffer(1, gpu_grass.buffer.slice(..));

            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..gpu_grass.length as u32);
                }
                GpuBufferInfo::NonIndexed => {
                    pass.draw(0..gpu_mesh.vertex_count, 0..gpu_grass.length as u32);
                }
            }
        }
        
        RenderCommandResult::Success
    }
}