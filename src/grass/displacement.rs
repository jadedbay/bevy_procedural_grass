use bevy::{prelude::*, render::{render_resource::TextureFormat, primitives::Aabb}, utils::HashSet, math::Vec3A};
use image::Rgba;
use super::chunk::GrassChunks;

#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Component, Default)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct GrassDisplacer {
    pub size: i32,
    pub base_offset: Vec3,
}

#[derive(Component, Clone, Default)]
pub struct GrassDisplacementImage {
    pub xz_image: Handle<Image>,
    pub xy_image: Handle<Image>,
    pub xz_edited_pixels: HashSet<(u32, u32)>,
    pub xy_edited_pixels: HashSet<(u32, u32)>,
}

impl GrassDisplacementImage {
    pub fn new(xz_image: Handle<Image>, xy_image: Handle<Image>) -> Self {
        Self {
            xz_image,
            xy_image,
            xz_edited_pixels: HashSet::new(),
            xy_edited_pixels: HashSet::new(),
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

            {
            let xz_image_handle = chunks.chunks.get(&chunk.0).unwrap().1.xz_image.clone();
            let image = images.get_mut(xz_image_handle.clone()).unwrap();
            let dynamic_image = image.clone().try_into_dynamic().unwrap();
            let mut buffer = dynamic_image.into_rgba8();

            if grass_timer.elapsed >= 0.02 {
                for (x, y) in chunks.chunks.get_mut(&chunk.0).unwrap().1.xz_edited_pixels.clone() {
                    let pixel = buffer.get_pixel_mut(x, y);
                    if pixel[3] > 2 {
                        pixel[3] = (pixel[3] as f32 * 0.8).round() as u8;
                    } else {
                        *pixel = Rgba([0, 0, 0, 0]);
                        chunks.chunks.get_mut(&chunk.0).unwrap().1.xz_edited_pixels.remove(&(x, y));
                    }
                }
            }

            for (object_transform, displacer) in object_query.iter() {
                let object_pos = object_transform.translation + displacer.base_offset;
                if (object_pos.x >= aabb.center.x - (aabb.half_extents.x + offset) && object_pos.x <= aabb.center.x + (aabb.half_extents.x + offset)) &&
                (object_pos.y >= aabb.center.y - (aabb.half_extents.y + offset) && object_pos.y <= aabb.center.y + (aabb.half_extents.y + offset)) &&
                (object_pos.z >= aabb.center.z - (aabb.half_extents.z + offset) && object_pos.z <= aabb.center.z + (aabb.half_extents.z + offset)) {
                    let xz_pos = Vec2::new(object_pos.x - aabb.center.x + aabb.half_extents.x, object_pos.z - aabb.center.z + aabb.half_extents.z);
                    let uv = xz_pos / chunk_size as f32;

                    let size = displacer.size;
                    let (width, height) = (image.width(), image.height());
                    let center_x = (uv.x * width as f32) as i32;
                    let center_y = (uv.y * height as f32) as i32;
                    let radius_squared = (size as f32 / 2.0).powi(2);

                    let start_x = center_x - size;
                    let start_y = center_y - size;
                    let end_x = center_x + size;
                    let end_y = center_y + size;
                    
                    for x in start_x..end_x {
                        for y in start_y..end_y {
                            if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                                continue;
                            }
                            if ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)) <= radius_squared {
                                let pixel = &mut buffer.get_pixel_mut(x as u32, y as u32).0;
                                let distance = ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)).sqrt();
                                let max_distance = size as f32 / 2.0;
                                let alpha = ((1.0 - (distance / max_distance)) * 255.0) as u8;
                                
                                if pixel[3] >= 10 {
                                    pixel[3] = std::cmp::max(pixel[3], alpha);
                                } else {
                                    let direction = Vec2::new(x as f32 - center_x as f32, y as f32 - center_y as f32).normalize();
                                    *pixel = [
                                        ((direction.x * 0.5 + 0.5) * 255.0) as u8,
                                        ((direction.y * 0.5 + 0.5) * 255.0) as u8,
                                        0,
                                        alpha,
                                    ];
                                    chunks.chunks.get_mut(&chunk.0).unwrap().1.xz_edited_pixels.insert((x as u32, y as u32));
                                }
                            }
                        }
                    }
                }
            }
            *image = Image::from_dynamic(buffer.into(), true)
                .convert(TextureFormat::Rgba8UnormSrgb)
                .unwrap();
        }

            let xy_image = images.get_mut(chunks.chunks.get(&chunk.0).unwrap().1.xy_image.clone()).unwrap();
            let dynamic_image = xy_image.clone().try_into_dynamic().unwrap();
            let mut buffer = dynamic_image.into_rgba8();

            if grass_timer.elapsed >= 0.02 {
                for (x, y) in chunks.chunks.get_mut(&chunk.0).unwrap().1.xy_edited_pixels.clone() {
                    let pixel = buffer.get_pixel_mut(x, y);
                    if pixel[3] > 2 {
                        pixel[3] = (pixel[3] as f32 * 0.8).round() as u8;
                    } else {
                        *pixel = Rgba([0, 0, 0, 0]);
                        chunks.chunks.get_mut(&chunk.0).unwrap().1.xy_edited_pixels.remove(&(x, y));
                    }
                }
            }

            for (object_transform, displacer) in object_query.iter() {
                let object_pos = object_transform.translation + displacer.base_offset;
                if (object_pos.x >= aabb.center.x - (aabb.half_extents.x + offset) && object_pos.x <= aabb.center.x + (aabb.half_extents.x + offset)) &&
                (object_pos.y >= aabb.center.y - (aabb.half_extents.y + offset) && object_pos.y <= aabb.center.y + (aabb.half_extents.y + offset)) &&
                (object_pos.z >= aabb.center.z - (aabb.half_extents.z + offset) && object_pos.z <= aabb.center.z + (aabb.half_extents.z + offset)) {
                    let xy_pos = Vec2::new(object_pos.x - aabb.center.x + aabb.half_extents.x, object_pos.y - aabb.center.y + aabb.half_extents.y);
                    let uv = xy_pos / chunk_size as f32;

                    let size = displacer.size;
                    let (width, height) = (xy_image.width(), xy_image.height());
                    let center_x = (uv.x * width as f32) as i32;
                    let center_y = (uv.y * height as f32) as i32;
                    let radius_squared = (size as f32 / 2.0).powi(2);

                    let start_x = center_x - size;
                    let start_y = center_y - size;
                    let end_x = center_x + size;
                    let end_y = center_y + size;
                    
                    for x in start_x..end_x {
                        for y in start_y..end_y {
                            if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
                                continue;
                            }
                            if ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)) <= radius_squared {
                                let pixel = &mut buffer.get_pixel_mut(x as u32, y as u32).0;
                                let distance = ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)).sqrt();
                                let max_distance = size as f32 / 2.0;
                                let alpha = ((1.0 - (distance / max_distance)) * 255.0) as u8;

                                if pixel[3] >= 10 {
                                    pixel[3] = std::cmp::max(pixel[3], alpha);
                                } else {
                                    let direction = Vec2::new(x as f32 - center_x as f32, y as f32 - center_y as f32).normalize();
                                    *pixel = [
                                        ((direction.x * 0.5 + 0.5) * 255.0) as u8,
                                        ((direction.y * 0.5 + 0.5) * 255.0) as u8,
                                        0,
                                        alpha,
                                    ];
                                    chunks.chunks.get_mut(&chunk.0).unwrap().1.xy_edited_pixels.insert((x as u32, y as u32));
                                }
                            }
                        }
                    }
                }
            }
            *xy_image = Image::from_dynamic(buffer.into(), true)
                .convert(TextureFormat::Rgba8UnormSrgb)
                .unwrap();
        }       
    }
    if grass_timer.elapsed >= 0.02 {
        grass_timer.elapsed = 0.;
    }
}