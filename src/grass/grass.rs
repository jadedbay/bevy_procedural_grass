use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, render_asset::{RenderAsset, PrepareAssetError}, renderer::RenderDevice, texture::{ImageType, CompressedImageFormats}}, ecs::system::{lifetimeless::SRes, SystemParamItem}, pbr::wireframe::Wireframe};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use rand::Rng;

use crate::grass::extract::{GrassInstanceData, InstanceData};

use super::{extract::{GrassColorData, WindData, LightData, BladeData}, wind::{Wind, WindMap}};

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub grass_entity: Option<Entity>,
    #[reflect(ignore)]
    pub grass_handle: Option<Handle<GrassInstanceData>>,
    #[reflect(ignore)]
    pub wind_map_handle: Handle<Image>,
    pub density: u32,
    pub color: GrassColor,
    pub blade: Blade,
    pub wind: Wind,
    pub regenerate: bool,
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Blade {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub bend: f32,
}

impl Default for Blade {
    fn default() -> Self {
        Self {
            length: 1.5,
            width: 1.,
            tilt: 0.5,
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
                grass_asset.remove(grass.grass_handle.clone().unwrap());
                let handle = generate_grass_data(&mut grass, transform, mesh, &mut grass_asset);
                grass.grass_handle = Some(handle.clone());
                commands.entity(grass_entity).insert(handle);
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
                .insert(WindData::from(grass.wind.clone()))
                .insert(BladeData::from(grass.blade.clone()));
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
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (transform, mut grass, mesh_handle) in query.iter_mut() {
        spawn_grass(&mut commands, transform, &mut grass, meshes.get(mesh_handle).unwrap(), &mut grass_asset);
    }
}

pub fn generate_grass_data(
    grass: &mut Grass,
    transform: &Transform,
    mesh: &Mesh,
    grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
) -> Handle<GrassInstanceData> {
    let mut data: Vec<InstanceData> = vec![];
    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            if let Some(indices) = mesh.indices() {
                let mut triangle = Vec::new();
                data = indices.iter().filter_map(|index| {
                    triangle.push(index);
                    if triangle.len() == 3 {
                        let result = {
                            // Calculate the area of the triangle
                            let v0 = Vec3::from(positions[triangle[0] as usize]) * transform.scale;
                            let v1 = Vec3::from(positions[triangle[1] as usize]) * transform.scale;
                            let v2 = Vec3::from(positions[triangle[2] as usize]) * transform.scale;

                            let normal = (v1 - v0).cross(v2 - v0).normalize();
        
                            let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
        
                            // Scale the density by the area of the triangle
                            let scaled_density = (grass.density as f32 * area).ceil() as u32;
        
                            (0..scaled_density).filter_map(|_| {
                                let mut rng = rand::thread_rng();
        
                                // Generate random barycentric coordinates
                                let r1 = rng.gen::<f32>().sqrt();
                                let r2 = rng.gen::<f32>();
                                let barycentric = Vec3::new(1.0 - r1, r1 * (1.0 - r2), r1 * r2);
        
                                // Calculate the position of the blade using the barycentric coordinates
                                let position = v0 * barycentric.x + v1 * barycentric.y + v2 * barycentric.z;
                            
                                let uv0 = Vec2::from(uvs[triangle[0] as usize]);
                                let uv1 = Vec2::from(uvs[triangle[1] as usize]);
                                let uv2 = Vec2::from(uvs[triangle[2] as usize]);
                                let uv = uv0 * barycentric.x + uv1 * barycentric.y + uv2 * barycentric.z;

                                Some(InstanceData {
                                    position,
                                    normal,
                                    uv,
                                })
                            }).collect::<Vec<_>>()
                        };
                        triangle.clear();
                        Some(result)
                    } else {
                        None
                    }
                }).flatten().collect();
            }
        }
    }
    dbg!(data.len());

    grass_asset.add(GrassInstanceData(data))
}

pub fn spawn_grass(
    commands: &mut Commands,
    transform: &Transform,
    grass: &mut Grass,
    mesh: &Mesh,
    grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
) {
    let handle = generate_grass_data(grass, transform, mesh, grass_asset);
    grass.grass_handle = Some(handle.clone());

    let grass_entity = commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        handle,
        GrassColorData::from(grass.color),
        WindData::from(grass.wind),
        BladeData::from(grass.blade),
        LightData::default(),
        WindMap {
            wind_map: grass.wind_map_handle.clone(),
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


pub struct GrassDataBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

impl RenderAsset for GrassInstanceData {
    type ExtractedAsset = GrassInstanceData;
    type PreparedAsset = GrassDataBuffer;
    type Param = SRes<RenderDevice>;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        dbg!("extract");
        GrassInstanceData(self.0.clone())
    }

    fn prepare_asset(
            extracted_asset: Self::ExtractedAsset,
            param: &mut SystemParamItem<Self::Param>,
        ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let render_device = param;

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(extracted_asset.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        
        Ok(GrassDataBuffer {
            buffer,
            length: extracted_asset.len(),
        })
    }
}