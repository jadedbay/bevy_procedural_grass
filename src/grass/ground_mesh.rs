use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, mesh::{Indices, VertexAttributeValues}, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use super::{Grass, GrassGround};

#[derive(Component, Clone)]
pub struct GroundMesh {
    pub positions_buffer: Buffer,
    pub indices_buffer: Buffer,
    pub triangle_count: usize,
}

pub(crate) fn prepare_ground_mesh(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass)>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass) in grass_query.iter() {
        let mesh = meshes.get(ground_query.get(grass.ground_entity.unwrap()).unwrap()).unwrap();

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => {
                warn!("Mesh does not contain positions, not generating grass.");
                return;
            },
        };
        let padded_positions: Vec<[f32; 4]> = positions.iter().map(|x| [x[0], x[1], x[2], 0.0]).collect();

        let indices = match mesh.indices() {
            Some(Indices::U32(indices)) => indices,
            _ => {
                warn!("Mesh does not contain indices");
                return;
            }, 
        };

        let positions_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("ground_mesh_positions"),
                contents: bytemuck::cast_slice(padded_positions.as_slice()),
                usage: BufferUsages::STORAGE,
            }
        );

        let indices_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("ground_mesh_indices"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::STORAGE,
            }
        );

        commands.entity(entity).insert(GroundMesh {
            positions_buffer,
            indices_buffer,
            triangle_count: indices.len() / 3,
        });
    }
}

impl ExtractComponent for GroundMesh {
    type QueryData = &'static GroundMesh;
    type QueryFilter = ();
    type Out = GroundMesh;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
       Some(item.clone()) 
    }
}
