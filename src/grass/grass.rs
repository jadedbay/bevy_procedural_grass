use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues}, utils::HashMap};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use rand::Rng;

use crate::render::{
    instance::{GrassInstanceData, GrassData},
    extract::{GrassColorData, WindData, BladeData}
};

use super::{wind::{WindMap, GrassWind}, chunk::{GrassChunks, GrassChunkHandles}};

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub grass_entity: Option<Entity>,
    pub density: u32,
    pub chunk_size: f32,
    pub color: GrassColor,
    pub blade: Blade,
    pub regenerate: bool,
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Blade {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub tilt_variance: f32,
    pub bend: f32,
}

impl Default for Blade {
    fn default() -> Self {
        Self {
            length: 1.5,
            width: 1.,
            tilt: 0.5,
            tilt_variance: 0.2,
            bend: 0.5,
        }
    }
}

pub fn update_grass_data(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Grass, &Handle<Mesh>), Changed<Grass>>,
    meshes: Res<Assets<Mesh>>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for (transform, mut grass, mesh_handle) in query.iter_mut() {
        if grass.regenerate {
            if let (Some(grass_entity), Some(mesh)) = (grass.grass_entity, meshes.get(mesh_handle)) {
                commands.entity(grass_entity).insert(generate_grass_data(&mut grass, transform, mesh, &mut grass_asset));
            }

            grass.regenerate = false;
        }
    }
}

pub fn update_grass_params(
    mut commands: Commands,
    query: Query<&Grass, Changed<Grass>>,
) {
    for grass in query.iter() {
        if let Some(grass_entity) = grass.grass_entity {
            commands.entity(grass_entity)
                .insert(GrassColorData::from(grass.color.clone()))
                .insert(BladeData::from(grass.blade.clone()));
        }
    }
}

pub fn load_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Grass, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
    wind: Res<GrassWind>,
) {
    for (entity, transform, mut grass, mesh_handle) in query.iter_mut() {
        spawn_grass(&mut commands, transform, &entity, &mut grass, meshes.get(mesh_handle).unwrap(), &mut grass_asset, &wind);
    }
}

pub fn generate_grass_data(
    grass: &mut Grass,
    transform: &Transform,
    mesh: &Mesh,
    _grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
) -> GrassChunks {
    let mut chunks: HashMap<(i32, i32, i32), GrassInstanceData> = HashMap::new();

    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            if let Some(indices) = mesh.indices() {
                let mut triangle = Vec::new();
                for index in indices.iter() {
                    triangle.push(index);
                    if triangle.len() == 3 {
                        let _result: Vec<GrassData> = {
                            let v0 = Vec3::from(positions[triangle[0] as usize]) * transform.scale;
                            let v1 = Vec3::from(positions[triangle[1] as usize]) * transform.scale;
                            let v2 = Vec3::from(positions[triangle[2] as usize]) * transform.scale;

                            let normal = (v1 - v0).cross(v2 - v0).normalize();
        
                            let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
        
                            let scaled_density = (grass.density as f32 * area).ceil() as u32;
        
                            (0..scaled_density).filter_map(|_| {
                                let mut rng = rand::thread_rng();
        
                                let r1 = rng.gen::<f32>().sqrt();
                                let r2 = rng.gen::<f32>();
                                let barycentric = Vec3::new(1.0 - r1, r1 * (1.0 - r2), r1 * r2);
        
                                let position = v0 * barycentric.x + v1 * barycentric.y + v2 * barycentric.z;
                            
                                let uv0 = Vec2::from(uvs[triangle[0] as usize]);
                                let uv1 = Vec2::from(uvs[triangle[1] as usize]);
                                let uv2 = Vec2::from(uvs[triangle[2] as usize]);
                                let uv = uv0 * barycentric.x + uv1 * barycentric.y + uv2 * barycentric.z;

                                let chunk_coords = (
                                    (position.x / grass.chunk_size).floor() as i32,
                                    (position.y / grass.chunk_size).floor() as i32,
                                    (position.z / grass.chunk_size).floor() as i32,
                                );

                                let instance = GrassData {
                                    position,
                                    normal,
                                    uv,
                                };

                                // Add instance to the appropriate chunk
                                chunks.entry(chunk_coords).or_insert_with(|| {GrassInstanceData(Vec::new())}).0.push(instance);

                                None
                            }).collect::<Vec<_>>()
                        };
                        triangle.clear();
                    }
                }
            }
        }
    }

    GrassChunks {
        chunks,
        chunk_size: grass.chunk_size,
        ..default()
    }
}

pub fn spawn_grass(
    commands: &mut Commands,
    transform: &Transform,
    entity: &Entity, 
    grass: &mut Grass,
    mesh: &Mesh,
    grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
    wind: &Res<GrassWind>,
) {
    let grass_handles = generate_grass_data(grass, transform, mesh, grass_asset);
    commands.entity(*entity).insert(grass_handles);

    let grass_entity = commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        GrassChunkHandles::default(),
        GrassColorData::from(grass.color),
        WindData::from(wind.wind_data),
        BladeData::from(grass.blade),
        WindMap {
            wind_map: wind.wind_map.clone()
        },
        NoFrustumCulling,
    )).id();

    grass.grass_entity = Some(grass_entity);
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct GrassColor {
    pub ao: Color,
    pub color_1: Color,
    pub color_2: Color,
    pub tip: Color,
}

impl Default for GrassColor {
    fn default() -> Self {
        Self {
            ao: [0.01, 0.02, 0.05, 1.0].into(),
            color_1: [0.1, 0.23, 0.09, 1.0].into(),
            color_2: [0.12, 0.39, 0.15, 1.0].into(),
            tip: [0.7, 0.7, 0.7, 1.0].into(),
        }
    }
}