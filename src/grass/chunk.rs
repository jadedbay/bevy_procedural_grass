
use bevy::{math::bounding::{Aabb2d, BoundingVolume}, prelude::*, render::{mesh::{Indices, VertexAttributeValues}, primitives::{Aabb, Frustum}, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, ShaderType}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};
use super::{Grass, GrassGround};
use crate::{grass::GrassGpuInfo, prefix_sum::calculate_workgroup_counts, render::instance::GrassInstanceData, util::aabb::triangle_intersects_aabb};

#[derive(Component, Clone)]
pub struct GrassChunks(pub HashMap<UVec2, (GrassChunk, bool)>);

#[derive(Component, Debug, Clone)]
pub struct GrassChunk {
    pub aabb: Aabb2d,
    pub aabb_buffer: Buffer,
    pub instance_buffer: Buffer,
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass, &Handle<Mesh>)>,
    ground_query: Query<&Handle<Mesh>, With<GrassGround>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass, mesh_handle) in grass_query.iter() {
        let mut grass_chunks = GrassChunks(HashMap::new());
        let mut chunks = Vec::new();

        let mesh = meshes.get(ground_query.get(grass.ground_entity.unwrap()).unwrap()).unwrap();
        let mesh_aabb = mesh.compute_aabb().unwrap();
        let mesh_aabb2d = Aabb2d::new(mesh_aabb.center.xz(), mesh_aabb.half_extents.xz());

        let chunk_size = (mesh_aabb2d.max - mesh_aabb2d.min) / (grass.chunk_count.as_vec2());

        let workgroup_count = (grass.density as f32 / (grass.chunk_count.x * grass.chunk_count.y) as f32).ceil() as usize;
        let instance_count = workgroup_count * 512;

        dbg!(instance_count);

        let (scan_workgroup_count, scan_groups_workgroup_count) = calculate_workgroup_counts(instance_count as u32);

        for x in 0..grass.chunk_count.x { 
            for z in 0..grass.chunk_count.y {
                let chunk_min = mesh_aabb2d.min + Vec2::new(x as f32, z as f32) * chunk_size;
                let chunk_max = chunk_min + chunk_size; 
                let aabb = Aabb2d {
                    min: chunk_min,
                    max: chunk_max,
                };

                // grass_chunks.0.insert(
                //     UVec2::new(x as u32, z as u32), 
                //     (
                //         GrassChunk { 
                //             aabb,
                //             aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                //                 label: Some("aabb_buffer"),
                //                 contents: bytemuck::cast_slice(&[Aabb2dGpu::from(aabb)]),
                //                 usage: BufferUsages::UNIFORM,
                //             }),
                //             instance_buffer: render_device.create_buffer(&BufferDescriptor {
                //                 label: Some("instance_buffer"),
                //                 size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                //                 usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                //                 mapped_at_creation: false, 
                //             }),                 
                //         },
                //         false,
                //     )
                // );

                let chunk = commands.spawn(
                    (
                        GrassChunk { 
                            aabb,
                            aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                                label: Some("aabb_buffer"),
                                contents: bytemuck::cast_slice(&[Aabb2dGpu::from(aabb)]),
                                usage: BufferUsages::UNIFORM,
                            }),
                            instance_buffer: render_device.create_buffer(&BufferDescriptor {
                                label: Some("instance_buffer"),
                                size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                                usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                                mapped_at_creation: false, 
                            }),                 
                        },
                        GrassGpuInfo {
                            aabb: mesh_aabb2d,
                            instance_count,
                            workgroup_count: workgroup_count as u32,
                            scan_groups_workgroup_count,
                            scan_workgroup_count,
                        },
                        grass.clone(),
                        mesh_handle.clone(),
                        SpatialBundle::default(),
                        NoFrustumCulling,
                    )
                ).id();

                chunks.push(chunk);
            }
        }
        commands.entity(entity).push_children(chunks.as_slice());
    }
}


pub(crate) fn distance_cull_chunks(
    mut query: Query<(&GrassChunk, &mut Visibility)>,
    camera_query: Query<&Transform>,
) {
    for (chunk, mut visibility) in query.iter_mut() {
        for transform in camera_query.iter() {
            if (chunk.aabb.center() - transform.translation.xz()).length() < 100.0 {
                *visibility = Visibility::Visible
            } else {
                *visibility = Visibility::Hidden
            }
        }
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub(crate) struct Aabb2dGpu {
    min: Vec2,
    max: Vec2,
}

impl From<Aabb2d> for Aabb2dGpu {
    fn from(aabb: Aabb2d) -> Self {
        Self {
            min: aabb.min,
            max: aabb.max,
        }
    }
}
