use bevy::{prelude::*, render::{render_resource::{Extent3d, TextureDimension, TextureFormat}, extract_component::ExtractComponent, primitives::Aabb, mesh::VertexAttributeValues}, ecs::query::QueryItem};

use super::grass::Grass;

#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};


#[derive(Component, Default, Clone, Copy)]
pub struct GrassInteractable {
    pub position: Vec3,
}

impl ExtractComponent for GrassInteractable {
    type Query = (&'static GrassInteractable, &'static Transform);
    type Filter = ();
    type Out = GrassInteractable;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(GrassInteractable {
            position: item.1.translation
        })
    }
}

#[derive(Component, Clone, Default)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct GrassInterableTarget {
    dimension: GrassInteractableDimension,
    pub image: Handle<Image>,
}

impl ExtractComponent for GrassInterableTarget {
    type Query = &'static GrassInterableTarget;
    type Filter = ();
    type Out = GrassInterableTarget;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

pub(crate) fn create_interactable_image(
    mut query: Query<&mut GrassInterableTarget>,
    asset_server: Res<AssetServer>,
) {
    for mut grass in query.iter_mut() {
        grass.image = asset_server.add(
            Image::new_fill(
                Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1,
                }, 
                TextureDimension::D2, 
                &[0, 0, 0, 0], 
                TextureFormat::Rgba8UnormSrgb,
            )
        );
    }
}

pub(crate) fn grass_interact(
    grass_query: Query<(&GrassInterableTarget, &Grass)>,
    object_query: Query<&Transform, With<GrassInteractable>>,
    mesh_entity_query: Query<(&Transform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (interactable, grass) in grass_query.iter() {
        for object_transform in object_query.iter() {
            let (mesh_transform, mesh_handle) = mesh_entity_query.get(grass.entity.unwrap()).unwrap();
            let mesh = meshes.get(mesh_handle).unwrap();
            let mesh_space = mesh_transform.compute_matrix().inverse();
            let local_pos = mesh_space.transform_point3(object_transform.translation);
            let image = images.get_mut(&interactable.image).unwrap();
            let dynamic_image = image.clone().try_into_dynamic().unwrap();
            let mut buffer = dynamic_image.into_rgba8();

            let clear_start = Instant::now();
            for x in 0..buffer.width() {
                for y in 0..buffer.height() {
                    let pixel = buffer.get_pixel_mut(x, y);
                    if pixel[3] > 0 {
                        pixel[3] -= 1;
                    }
                }
            }
            let clear_duration = clear_start.elapsed();
            println!("Time elapsed in ray.intersect_mesh(mesh) is: {:?}", clear_duration);

            let ray = Ray::new(local_pos.xyz(), Vec3::new(0., -1., 0.));

            let start = Instant::now();
            if let Some(intersection) = ray.intersect_mesh(mesh) {
                let uv = intersection.0;
                let size = 35;
                let (width, height) = (image.width(), image.height());
                let center_x = (uv.x * width as f32) as u32;
                let center_y = (uv.y * height as f32) as u32;
                let radius_squared = (size as f32 / 2.0).powi(2);

                let start_x = center_x.saturating_sub(size);
                let start_y = center_y.saturating_sub(size);
                let end_x = (center_x.saturating_add(size)).min(width);
                let end_y = (center_y.saturating_add(size)).min(height);

                for x in start_x..end_x {
                    for y in start_y..end_y {
                        if ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)) <= radius_squared {
                            let pixel = &mut buffer.get_pixel_mut(x, y).0;
                            if pixel[3] >= 1 {
                                let distance = ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)).sqrt();
                                let max_distance = size as f32 / 2.0;
                                let alpha = ((1.0 - (distance / max_distance)) * 255.0) as u8;
                                pixel[3] = alpha;
                            } else {
                                let direction = Vec2::new(x as f32 - center_x as f32, y as f32 - center_y as f32).normalize();
                                let distance = ((x as f32 - center_x as f32).powi(2) + (y as f32 - center_y as f32).powi(2)).sqrt();
                                let max_distance = size as f32 / 2.0;
                                let alpha = ((1.0 - (distance / max_distance)) * 255.0) as u8;

                                *pixel = [
                                    ((direction.x * 0.5 + 0.5) * 255.0) as u8,
                                    ((direction.y * 0.5 + 0.5) * 255.0) as u8,
                                    0,
                                    alpha,
                                ];
                            }
                        }
                    }
                }
                *image = Image::from_dynamic(buffer.into(), true).convert(TextureFormat::Rgba8UnormSrgb).unwrap();
            }

            let duration = start.elapsed();
            //println!("Time elapsed in ray.intersect_mesh(mesh) is: {:?}", duration);
        }
    }
}

pub fn update_interaction_image(
    mut query: Query<&mut GrassInterableTarget>,
) {
    for mut grass in query.iter_mut() {
        grass.as_mut();
    }
}

use std::time::Instant;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub enum GrassInteractableDimension {
    D2,
    D3,
}

impl Default for GrassInteractableDimension {
    fn default() -> Self {
        Self::D2
    }
}

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
        }
    }

    pub fn intersect_mesh(&self, mesh: &Mesh) -> Option<(Vec2, f32)> {
        if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                if let Some(indices) = mesh.indices() {
                    let mut triangle = Vec::new();
                    for index in indices.iter() {
                        triangle.push(index);
                        if triangle.len() == 3 {
                            let v0 = positions[triangle[0] as usize].into();
                            let v1 = positions[triangle[1] as usize].into();
                            let v2 = positions[triangle[2] as usize].into();

                            if let Some((barycentric, distance)) = self.intersects_triangle(v0, v1, v2) {
                                let uv0 = Vec2::from(uvs[triangle[0] as usize]);
                                let uv1 = Vec2::from(uvs[triangle[1] as usize]);
                                let uv2 = Vec2::from(uvs[triangle[2] as usize]);

                                let uv = uv0 * barycentric.x + uv1 * barycentric.y + uv2 * barycentric.z;
                                return Some((uv.into(), distance));
                            }
                            triangle.clear();
                        }
                    }
                }
            }
        }

        None
    }

    // Möller–Trumbore ray-triangle intersection algorithm
    pub fn intersects_triangle(&self, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<(Vec3, f32)> {

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = self.direction.cross(edge2);
        let a = edge1.dot(h);

        if a > -f32::EPSILON && a < f32::EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = self.origin - v0;
        let u = f * s.dot(h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * self.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);

        if t > f32::EPSILON {
            return Some((Vec3::new(1.0 - u - v, u, v), t));
        }

        None
    }
}