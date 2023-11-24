use bevy::{prelude::*, render::{render_phase::{SetItemPipeline, PhaseItem, RenderCommand, TrackedRenderPass, RenderCommandResult}, render_asset::RenderAssets, mesh::GpuBufferInfo}, pbr::{SetMeshViewBindGroup, SetMeshBindGroup, RenderMeshInstances}, ecs::system::{lifetimeless::{SRes, Read}, SystemParamItem}};

use super::{prepare::{ColorBindGroup, WindBindGroup, BladeBindGroup, WindMapBindGroup, InstanceBuffer}, extract::GrassInstanceData, compute_pipeline::GrassDataBindGroup, chunk::{GrassChunks, GrassToDraw}};

pub type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetColorBindGroup<2>,
    SetWindBindGroup<3>,
    SetBladeBindGroup<4>,
    SetWindMapBindGroup<5>,
    DrawGrassInstanced,
);

pub struct SetColorBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetColorBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<ColorBindGroup>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w ColorBindGroup>,
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

pub struct SetWindBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetWindBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<WindBindGroup>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w WindBindGroup>,
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

pub struct SetBladeBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetBladeBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<BladeBindGroup>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w BladeBindGroup>,
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

pub struct SetWindMapBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetWindMapBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<WindMapBindGroup>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w WindMapBindGroup>,
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
    type ItemWorldQuery = Read<GrassToDraw>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        chunks: &'w GrassToDraw,
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