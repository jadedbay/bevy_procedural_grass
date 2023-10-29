use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues}};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use rand::Rng;

use crate::grass::extract::{GrassInstanceData, InstanceData};

use super::{extract::{GrassColorData, WindData, LightData}, wind::Wind};

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub grass_entity: Option<Entity>,
    pub density: u32,
    pub offset_strength: f32,
    pub color: GrassColor,
    pub wind: Wind,
    pub regenerate: bool,
}

pub fn update_grass_data(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Grass, &Handle<Mesh>), Changed<Grass>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (transform, mut grass, mesh_handle) in query.iter_mut() {
        if grass.regenerate {
            if let (Some(grass_entity), Some(mesh)) = (grass.grass_entity, meshes.get(mesh_handle)) {
                commands.entity(grass_entity).insert(generate_grass_data(&mut grass, transform, mesh));
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
                .insert(WindData::from(grass.wind.clone()));
        }
    }
}

pub fn update_light(
    mut query: Query<&mut LightData>,
    light_query: Query<&Transform, With<DirectionalLight>>,
) {
    for mut light_data in query.iter_mut() {
        for transform in light_query.iter() {
            let direction = transform.rotation.to_euler(EulerRot::XYZ);

            light_data.direction = Vec3::new(direction.0, direction.1, direction.2);
        }
    }
}

pub fn load_grass(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Grass, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (transform, mut grass, mesh_handle) in query.iter_mut() {
        spawn_grass(&mut commands, transform, &mut grass, meshes.get(mesh_handle).unwrap());
    }
}

pub fn generate_grass_data(
    grass: &mut Grass,
    transform: &Transform,
    mesh: &Mesh,
) -> GrassInstanceData {
    let offset = grass.offset_strength;

    let mut data: Vec<InstanceData> = vec![];
    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        data = positions.iter().flat_map(|position| {
            (0..grass.density).map(move |_| {
                let mut rng = rand::thread_rng();
                let offset_x = rng.gen_range(-offset..offset);
                let offset_z = rng.gen_range(-offset..offset);
                let mut blade_position = Vec3::new(position[0], position[1], position[2]) * transform.scale;
                blade_position += Vec3::new(offset_x, 1.0, offset_z);

                InstanceData {
                    position: blade_position,
                    uv: Vec2::new(
                        (position[0] / transform.scale.x) + 0.5,
                        (position[2] / transform.scale.z) + 0.5,
                    ),
                }
            })
        }).collect();
    }
    dbg!(data.len());

    GrassInstanceData(data)
}

pub fn spawn_grass(
    commands: &mut Commands,
    transform: &Transform,
    grass: &mut Grass,
    mesh: &Mesh,
) {
    let grass_entity = commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        generate_grass_data(grass, transform, mesh),
        GrassColorData::from(grass.color),
        WindData::from(grass.wind),
        LightData::default(),
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