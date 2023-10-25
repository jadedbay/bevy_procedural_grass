use bevy::{prelude::*, render::{render_phase::{SetItemPipeline, PhaseItem, RenderCommand, TrackedRenderPass, RenderCommandResult}, render_asset::RenderAssets, mesh::GpuBufferInfo}, pbr::{SetMeshViewBindGroup, SetMeshBindGroup}, ecs::system::{lifetimeless::{SRes, Read}, SystemParamItem}};

use super::prepare::{ColorBindGroup, InstanceBuffer};

pub type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetColorBindGroup<2>,
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

pub struct DrawGrassInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<InstanceBuffer>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, instance_buffer): (&'w Handle<Mesh>, &'w InstanceBuffer),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
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