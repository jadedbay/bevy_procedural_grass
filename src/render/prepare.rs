use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, BufferBinding}, renderer::RenderDevice}};

use super::{mesh_asset::GrassBaseMesh, pipeline::GrassComputePipeline};

#[derive(Component)]
pub struct GrassComputeBindGroup {
    pub mesh_positions_bind_group: BindGroup,
}

pub(crate) fn prepare_compute_bind_groups(
    mut commands: Commands,
    grass_base_meshes: ResMut<RenderAssets<GrassBaseMesh>>,
    pipeline: Res<GrassComputePipeline>,
    query: Query<(Entity, &Handle<Mesh>)>,
    render_device: Res<RenderDevice>
) {
    let layout = pipeline.mesh_layout.clone();

    for (entity, mesh_handle) in query.iter() {
        let mesh_positions_bind_group = render_device.create_bind_group(
            Some("mesh_position_bind_group"),
            &layout,
            &BindGroupEntries::single(
                BufferBinding {
                    buffer: &grass_base_meshes.get(mesh_handle).unwrap().positions_buffer,
                    offset: 0,
                    size: None,
                }
            )
        );

        commands.entity(entity).insert(GrassComputeBindGroup {
            mesh_positions_bind_group,
        });
    }
}