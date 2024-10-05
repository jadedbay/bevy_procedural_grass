use bevy::{math::{bounding::Aabb2d, Affine3A}, prelude::*, render::{primitives::{Aabb, Frustum}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};

use crate::prelude::GrassMaterial;

use super::{chunk::{GrassChunk, GrassChunkBuffers}, config::GrassConfig, Grass, GrassGpuInfo};

#[derive(Component)]
pub(crate) struct GrassCullChunks(pub HashMap<UVec2, Entity>);

pub(crate) fn cull_chunks(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut q_grass: Query<(Entity, &Grass, &GrassGpuInfo, &mut GrassCullChunks, &Handle<Mesh>, &Handle<GrassMaterial>, &Visibility)> ,
    camera_query: Query<(&Transform, &Frustum)>,
    grass_config: Res<GrassConfig>,
) {
    for (entity, grass, gpu_info, mut cull_chunks, mesh, material, visibility) in &mut q_grass {
        let chunk_min = gpu_info.aabb.min;
        let chunk_max = chunk_min + gpu_info.chunk_size;
        
        let aabb = Aabb::from_min_max(
            Vec3::new(chunk_min.x, -grass.height_map.as_ref().unwrap().scale, chunk_min.y), 
            Vec3::new(chunk_max.x, grass.height_map.as_ref().unwrap().scale, chunk_max.y),
        );

        let mut new_chunks = Vec::new();
        
        for x in 0..grass.chunk_count.x {
            'chunk: for z in 0..grass.chunk_count.y { 
                let chunk_pos = UVec2::new(x, z);
                let world_pos = Vec3::new(x as f32 * gpu_info.chunk_size.x, 0.0, z as f32 * gpu_info.chunk_size.y);

                for (transform, frustum) in camera_query.iter() {

                    // TODO: Seperate distance/frustum culling and keep chunks in distance loaded but tell prepare to not create bind groups
                    if ((Vec3::from(aabb.center) + world_pos).xz() - transform.translation.xz()).length() < grass_config.cull_distance 
                    && frustum.intersects_obb(&aabb, &Affine3A::from_translation(world_pos), false, false) {
                        if !cull_chunks.0.contains_key(&chunk_pos) {
                            let chunk_aabb = Aabb2d {
                                min: aabb.min().xz() + world_pos.xz(),
                                max: aabb.max().xz() + world_pos.xz(),
                            };

                            let chunk_entity = commands.spawn((
                                GrassChunk {
                                    grass_entity: entity,
                                    aabb: chunk_aabb,
                                    instance_count: gpu_info.instance_count,
                                    scan_workgroup_count: gpu_info.scan_workgroup_count,
                                },
                                GrassChunkBuffers::create_buffers(
                                    &render_device,
                                    chunk_aabb,
                                    gpu_info.instance_count,
                                    gpu_info.scan_workgroup_count,
                                    grass_config.grass_shadows,
                                ),
                                mesh.clone(),
                                material.clone(),
                                SpatialBundle {
                                    visibility: *visibility,
                                    ..default()
                                },
                                NoFrustumCulling,
                            )).id();

                            cull_chunks.0.insert(chunk_pos, chunk_entity);
                            new_chunks.push(chunk_entity);
                        }

                        continue 'chunk;
                    } else {
                        if let Some(chunk_entity) = cull_chunks.0.remove(&chunk_pos) {
                            commands.entity(entity).remove_children(&[chunk_entity]);
                            commands.entity(chunk_entity).despawn();
                        }
                    }
                }
            }
        }
        
        commands.entity(entity).push_children(new_chunks.as_slice());
    }
}