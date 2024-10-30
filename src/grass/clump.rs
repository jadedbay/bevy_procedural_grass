use bevy::{math::bounding::Aabb2d, prelude::*, render::{extract_resource::ExtractResource, render_resource::{AsBindGroup, BindGroup, BindGroupEntries, Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}, utils::HashMap};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{render::pipeline::GrassComputePipeline, util::aabb::Aabb2dGpu};

#[derive(Resource, ExtractResource, Clone, Reflect)]
#[reflect(Resource)]
pub struct GrassClumpConfig {
    pub seed: u64,
    pub aabb: Aabb2d,
    pub count: UVec2,
}
impl Default for GrassClumpConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            aabb: Aabb2d {
                min: Vec2::new(-50.0, -50.0),
                max: Vec2::new(50.0, 50.0),
            },
            count: UVec2::new(40, 40),
        }
    }
}

impl GrassClumpConfig {
    fn generate_clumps(&self) -> GrassClumps {
        let cell_size = Vec2::new(
            (self.aabb.max.x - self.aabb.min.x) / self.count.x as f32,
            (self.aabb.max.y - self.aabb.min.y) / self.count.y as f32,
        );

        let mut rng = StdRng::seed_from_u64(self.seed);

        let mut random_points = Vec::new();
        let mut clumps = Vec::new();
        for x in 0..self.count.x {
            for y in 0..self.count.y {
                let x_range = self.aabb.min.x + x as f32 * cell_size.x..self.aabb.min.x + (x + 1) as f32 * cell_size.x;
                let y_range = self.aabb.min.y + y as f32 * cell_size.y..self.aabb.min.y + (y + 1) as f32 * cell_size.y;
                random_points.push(Vec2::new(rng.gen_range(x_range), rng.gen_range(y_range)));
                

                let facing = match rng.gen_range(1..=3) {
                    0 => GrassClumpDirection::In,
                    1 => GrassClumpDirection::Out,
                    2 => GrassClumpDirection::Random,
                    _ => {
                        let random_angle = rng.gen_range(0.0..std::f32::consts::TAU);
                        let random_direction = Vec2::new(random_angle.cos(), random_angle.sin());
                        GrassClumpDirection::Facing(random_direction)
                    }
                }.to_vec2();

                clumps.push(
                    GrassClump {
                        color: LinearRgba::rgb(1.0, 1.0, 1.0).to_vec4(),
                        facing,
                        length: rng.gen_range(0.7..1.2),
                        tilt: 0.8,
                    }
                )
            }
        }

        GrassClumps {
            cell_size,
            positions: random_points,
            params: clumps,
        }
    }
}

pub struct ClumpColors {
    colors: HashMap<Color, f32>,
}


pub enum GrassClumpDirection {
    In,
    Out,
    Random,
    Facing(Vec2),
}
impl GrassClumpDirection {
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Self::In => Vec2::new(2.0, 0.0),
            Self::Out => Vec2::new(3.0, 0.0),
            Self::Random => Vec2::new(4.0, 0.0),
            Self::Facing(direction) => direction.normalize(),
        }
    }
}



#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GrassClump {
    color: Vec4,
    facing: Vec2, 
    length: f32,
    tilt: f32,
}

#[derive(Resource, ExtractResource, Clone)]
pub struct GrassClumps {
    cell_size: Vec2,
    pub positions: Vec<Vec2>,
    pub params: Vec<GrassClump>,
}

pub(crate) fn clump_startup(
    mut commands: Commands,
    clump_config: Res<GrassClumpConfig>,
) {
    commands.insert_resource(clump_config.generate_clumps());
}

#[derive(Resource)]
pub struct GrassClumpsGpu {
    positions_buffer: Buffer,
    params_buffer: Buffer,
    pub bind_group: BindGroup,
}

pub(crate) fn prepare_clump(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    clumps: Res<GrassClumps>,
    clump_config: Res<GrassClumpConfig>,
    pipeline: Res<GrassComputePipeline>,
) {
    let aabb_buffer = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("clump_aabb_buffer"),
            contents: &bytemuck::cast_slice(&[Aabb2dGpu::from(clump_config.aabb)]),
            usage: BufferUsages::UNIFORM,
        }
    );
    let clump_size_buffer = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("clump_size_buffer"),
            contents: &bytemuck::cast_slice(&[clumps.cell_size]),
            usage: BufferUsages::UNIFORM,
        }
    );
    let positions_buffer = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("clump_positions_buffer"),
            contents: &bytemuck::cast_slice(clumps.positions.as_slice()),
            usage: BufferUsages::STORAGE,
        }
    );
    let params_buffer = render_device.create_buffer_with_data( 
        &BufferInitDescriptor {
            label: Some("clump_params_buffer"),
            contents: &bytemuck::cast_slice(clumps.params.as_slice()),
            usage: BufferUsages::STORAGE,
        }
    );
    let bind_group = render_device.create_bind_group(
        Some("clump_bind_group"),
        &pipeline.clump_layout,
        &BindGroupEntries::sequential((
            aabb_buffer.as_entire_binding(),
            clump_size_buffer.as_entire_binding(),
            positions_buffer.as_entire_binding(),
            params_buffer.as_entire_binding(),
        )) 
    );

    commands.insert_resource(GrassClumpsGpu {
        positions_buffer,
        params_buffer,
        bind_group,
    });
}