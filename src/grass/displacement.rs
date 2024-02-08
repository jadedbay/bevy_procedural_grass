use bevy::{prelude::*, render::{render_resource::{TextureFormat, Extent3d, TextureDimension}, primitives::Aabb}, utils::HashSet, math::Vec3A};
use image::{Rgba, ImageBuffer};
use super::chunk::GrassChunks;

#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Component, Default)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct GrassDisplacer {
    pub width: f32,
    pub height: f32,
    pub base_offset: Vec3,
}

#[derive(Component, Clone, Default)]
pub struct GrassDisplacementImage {
    pub image: Handle<Image>,
    image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>, 
    pub edited_pixels: HashSet<(u32, u32)>,
    pub height_edited_pixels: HashSet<(u32,  u32)>,
}

impl GrassDisplacementImage {
    pub fn new(image: Handle<Image>, resolution: u32) -> Self {
        Self {
            image,
            image_buffer: ImageBuffer::new(resolution, resolution),
            edited_pixels: HashSet::new(),
            height_edited_pixels: HashSet::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct GrassTimer {
    pub elapsed: f32
}

pub(crate) fn grass_displacement(
    time: Res<Time>,
    mut grass_timer: ResMut<GrassTimer>,
    mut grass_query: Query<&mut GrassChunks>,
    object_query: Query<(&Transform, &GrassDisplacer)>,
    mut images: ResMut<Assets<Image>>,
    mut gizmos: Gizmos,
) {
    grass_timer.elapsed += time.delta_seconds();
    for mut chunks in grass_query.iter_mut() {
        let loaded_chunks = chunks.loaded.clone();
        for chunk in loaded_chunks {
            let chunk_coord = chunk.0;
            let chunk_size = chunks.chunk_size;
            let aabb = Aabb {
                center: Vec3A::from(((chunk_coord.0 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.1 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.2 as f32 * chunk_size + chunk_size / 2.))),
                half_extents: Vec3A::splat(chunk_size as f32 / 2.0),
            };
            let offset = 2.;

            crate::util::draw_chunk(&mut gizmos, &chunk_coord, chunk_size);

            let image_handle = chunks.chunks.get(&chunk.0).unwrap().1.image.clone();

            let mut changed = false;
            
            if grass_timer.elapsed >= 0.02 {
                for (x, y) in chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.clone() {
                    let pixel = chunks.chunks.get_mut(&chunk.0).unwrap().1.image_buffer.get_pixel_mut(x, y);
                    if pixel[3] > 1 {
                        pixel[3] = (pixel[3] as f32 * 0.8).floor() as u8;
                    } else {
                        *pixel = Rgba([0, 0, 0, 0]);
                        chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.remove(&(x, y));
                    }
                    changed = true;
                }
            }
            
            for (object_transform, displacer) in object_query.iter() {
                let object_pos = object_transform.translation + displacer.base_offset;
                if (object_pos.x >= aabb.center.x - (aabb.half_extents.x + offset) && object_pos.x <= aabb.center.x + (aabb.half_extents.x + offset)) &&
                (object_pos.y >= aabb.center.y - (aabb.half_extents.y + offset) && object_pos.y <= aabb.center.y + (aabb.half_extents.y + offset)) &&
                (object_pos.z >= aabb.center.z - (aabb.half_extents.z + offset) && object_pos.z <= aabb.center.z + (aabb.half_extents.z + offset)) {
                    let image = images.get_mut(image_handle.clone()).unwrap();
                    
                    let pos = Vec3::new(object_pos.x - aabb.center.x + aabb.half_extents.x, object_pos.y - aabb.center.y + aabb.half_extents.y, object_pos.z - aabb.center.z + aabb.half_extents.z);
                    let uv = pos / chunk_size as f32;
                    
                    let size = displacer.width as f32;
                    let radius_squared = (size as f32 / 2.0).powi(2);
                    let (width, height) = (image.width(), image.height());

                    let center_x = uv.x * width as f32;
                    let center_y = uv.z * height as f32;
                    
                    let start_x = center_x - size;
                    let start_y = center_y - size;
                    let end_x = center_x + size;
                    let end_y = center_y + size;
                    
                    for x in start_x as i32..end_x as i32 {
                        for y in start_y as i32..end_y as i32 {
                            if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                                continue;
                            }
                            if ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)) <= radius_squared {
                                let pixel = &mut chunks.chunks.get_mut(&chunk.0).unwrap().1.image_buffer.get_pixel_mut(x as u32, y as u32).0;
                                let distance = ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)).sqrt();
                                let max_distance = size as f32 / 2.0;
                                let alpha = ((1.0 - (distance / max_distance)) * 255.0) as u8;
                                
                                if pixel[3] >= 5 {
                                    pixel[3] = std::cmp::max(pixel[3], alpha);
                                    pixel[2] = (uv.y * 255.0) as u8;
                                } else {
                                    let direction = Vec2::new(x as f32 - center_x as f32, y as f32 - center_y as f32).normalize();
                                    let angle = direction.y.atan2(direction.x);
                                    let angle_normalized = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                                    let angle_as_u8 = (angle_normalized * 255.0) as u8;
                                    pixel[0] = angle_as_u8;
                                    pixel[2] = (uv.y * 255.0) as u8;
                                    pixel[3] = std::cmp::max(pixel[3], alpha);
                                    chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.insert((x as u32, y as u32));
                                }
                            }
                            changed = true;
                        }
                    }
                }
            }
            
            if changed {
                
                let image = images.get_mut(image_handle).unwrap();
                *image = Image::new(
                    Extent3d {
                        width: image.width(),
                        height: image.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    chunks.chunks.get_mut(&chunk.0).unwrap().1.image_buffer.clone().into_raw(),
                    TextureFormat::Rgba8Unorm,
                );
            }
        }
    }
    if grass_timer.elapsed >= 0.02 {
        grass_timer.elapsed = 0.;
    }
}