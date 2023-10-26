use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use noise::NoiseFn;
use rand::Rng;

use crate::{terrain::component::Terrain, grass::extract::{GrassInstanceData, InstanceData}};

use super::extract::GrassColorData;

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub instance_data: GrassInstanceData,
    pub density: u32,
    pub color: GrassColor,
    pub regenerate: bool,
}

pub fn update_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Terrain, &mut Grass)>,
    grass_entity_query: Query<(Entity, &GrassId)>,
) {
    for (entity, transform, terrain, mut grass) in query.iter_mut() {
        if grass.regenerate {
            generate_grass_data(transform, terrain, &mut grass);
            for (grass_entity, grass_id) in grass_entity_query.iter() {
                if grass_id.0 == entity.index() {
                    commands.entity(grass_entity).insert(grass.instance_data.clone());
                }
            }

            grass.regenerate = false;
        }
    }
}

pub fn update_grass_color(
    mut commands: Commands,
    query: Query<(Entity, &Grass), Changed<Grass>>,
    grass_entity_query: Query<(Entity, &GrassId)>,
) {
    for (entity, grass) in query.iter() {
        for (grass_entity, grass_id) in grass_entity_query.iter() {
            if grass_id.0 == entity.index() {
                commands.entity(grass_entity).insert(GrassColorData::from(grass.color.clone()));
            }
        }
    }
}

pub fn load_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Terrain, &mut Grass)>
) {
    for (entity, transform, terrain, mut grass) in query.iter_mut() {
        generate_grass_data(transform, terrain, &mut grass);
        spawn_grass(&mut commands, entity, &grass);
    }
}

pub fn generate_grass_data(
    transform: &Transform,
    terrain: &Terrain, 
    grass: &mut Grass,
) {
    let size = transform.scale / 2.0;
    let density = grass.density;

    let rng = rand::thread_rng();

    let data: Vec<InstanceData> = 
    (-size.x as i32 * density as i32..=size.x as i32 * density as i32)
    .flat_map(|x| {
        let mut rng = rng.clone();
        (-size.z as i32 * density as i32..=size.z as i32 * density as i32)
        .map(move |z| {
            let offset_x = rng.gen_range(-0.5..0.5);
            let offset_z = rng.gen_range(-0.5..0.5);

            let pos_x = (x as f32 + offset_x) / density as f32;
            let pos_z = (z as f32 + offset_z) / density as f32;

            let mut y = 1.;
            if let Some(noise) = &terrain.noise {
                y += (noise::Perlin::new(noise.seed).get([(pos_x * noise.intensity / transform.scale.x) as f64, (pos_z * noise.intensity / transform.scale.z) as f64]) as f32) * terrain.get_height_scale();
            }

            InstanceData {
                position: Vec3::new(pos_x, y, pos_z),
                uv: Vec2::new(
                    (pos_x / transform.scale.x) + 0.5,
                    (pos_z / transform.scale.z) + 0.5,
                ),
            }
        })
    })
    .collect();
    dbg!(data.len());

    grass.instance_data = GrassInstanceData(data);
}

pub fn spawn_grass(
    commands: &mut Commands,
    entity: Entity,
    grass: &Grass,
) {
    commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        grass.instance_data.clone(),
        GrassColorData::from(grass.color),
        NoFrustumCulling,
        GrassId(entity.index())
    ));
}

#[derive(Component)]
pub struct GrassId(u32);

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
            color_1: [0.33, 0.57, 0.29, 1.0].into(),
            color_2: [0.08, 0.43, 0.29, 1.0].into(),
            tip: [1.0, 1.0, 1.0, 1.0].into(),
        }
    }
}