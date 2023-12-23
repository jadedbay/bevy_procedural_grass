use bevy::{prelude::*, render::{render_resource::TextureFormat, primitives::Aabb}, utils::HashSet, math::Vec3A};
use image::Rgba;
use super::chunk::GrassChunks;

#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Component, Default)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct GrassInteractable {
    pub size: i32
}

#[derive(Component, Clone, Default)]
pub struct GrassInteractableTarget {
    pub image: Handle<Image>,
    pub edited_pixels: HashSet<(u32, u32)>,
}

impl GrassInteractableTarget {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            edited_pixels: HashSet::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct GrassTimer {
    pub elapsed: f32
}

pub(crate) fn chunk_interact(
    time: Res<Time>,
    mut grass_timer: ResMut<GrassTimer>,
    mut grass_query: Query<&mut GrassChunks>,
    object_query: Query<(&Transform, &GrassInteractable)>,
    mut images: ResMut<Assets<Image>>,
) {
    grass_timer.elapsed += time.delta_seconds();
    for mut chunks in grass_query.iter_mut() {
        let loaded_chunks = chunks.loaded.clone();
        for chunk in loaded_chunks {
            let image = images.get_mut(chunks.chunks.get(&chunk.0).unwrap().1.image.clone()).unwrap();
            let dynamic_image = image.clone().try_into_dynamic().unwrap();
            let mut buffer = dynamic_image.into_rgba8();

            if grass_timer.elapsed >= 0.02 {
                for (x, y) in chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.clone() {
                    let pixel = buffer.get_pixel_mut(x, y);
                    if pixel[3] > 1 {
                        pixel[3] = (pixel[3] as f32 * 0.8).round() as u8;
                    } else if pixel[3] == 10 {
                        *pixel = Rgba([0, 0, 0, 0]);
                        chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.remove(&(x, y));
                    }
                }
            }

            for (object_transform, interact) in object_query.iter() {
                let chunk_coord = chunk.0;
                let chunk_size = chunks.chunk_size;
                let aabb = Aabb {
                    center: Vec3A::from(((chunk_coord.0 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.1 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.2 as f32 * chunk_size + chunk_size / 2.))),
                    half_extents: Vec3A::splat(chunk_size as f32 / 2.0),
                };
                let offset = 2.;

                let object_pos = object_transform.translation;
                if (object_pos.x >= aabb.center.x - (aabb.half_extents.x + offset) && object_pos.x <= aabb.center.x + (aabb.half_extents.x + offset)) &&
                (object_pos.y >= aabb.center.y - (aabb.half_extents.y + offset) && object_pos.y <= aabb.center.y + (aabb.half_extents.y + offset)) &&
                (object_pos.z >= aabb.center.z - (aabb.half_extents.z + offset) && object_pos.z <= aabb.center.z + (aabb.half_extents.z + offset)) {
                    let xz_pos = Vec2::new(object_pos.x - aabb.center.x + aabb.half_extents.x, object_pos.z - aabb.center.z + aabb.half_extents.z);
                    let uv = xz_pos / chunk_size as f32;

                    let size = interact.size;
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
                                    chunks.chunks.get_mut(&chunk.0).unwrap().1.edited_pixels.insert((x as u32, y as u32));
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
    }
    if grass_timer.elapsed >= 0.02 {
        grass_timer.elapsed = 0.;
    }
}