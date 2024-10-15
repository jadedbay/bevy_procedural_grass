#define_import_path bevy_procedural_grass

@group(2) @binding(100)
var<uniform> grass_material: GrassMaterial;
@group(2) @binding(101)
var grass_texture: texture_2d<f32>;
@group(2) @binding(102)
var wind_texture: texture_2d<f32>;

struct GrassInstance {
    position: vec4<f32>,
    chunk_uv: vec2<f32>,
    facing: vec2<f32>,
}

struct Aabb2d {
    min: vec2<f32>,
    max: vec2<f32>,
}

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
}

struct GrassMaterial {
    width: f32,
    curve: f32,
    midpoint: f32,
    roughness_variance: f32,
    reflectance_variance: f32,
    min_ao: f32,
    midrib_softness: f32,
    rim_position: f32,
    rim_softness: f32,
    width_normal_strength: f32,
    texture_strength: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) facing: vec2<f32>,
    @location(4) t: vec4<f32>,
}

const identity_matrix: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

fn rotate(v: vec3<f32>, direction: vec2<f32>) -> vec3<f32> {
    let angle = atan2(direction.y, direction.x);
    let rotation_matrix = mat3x3<f32>(
        cos(angle), 0.0, -sin(angle),
        0.0, 1.0, 0.0,
        sin(angle), 0.0, cos(angle)
    );

    return rotation_matrix * v;
}

fn rotate_x(v: vec3<f32>, angle: f32) -> vec3<f32> {
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    let rotation_matrix = mat3x3<f32>(
        1.0, 0.0, 0.0,
        0.0, cos_angle, -sin_angle,
        0.0, sin_angle, cos_angle
    );
    return rotation_matrix * v;
}

fn quadratic_bezier(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> vec2<f32> {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    var p = uu * p0;
    p = p + 2.0 * u * t * p1;
    p = p + tt * p2;

    return p;
}

fn bezier_tangent(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> vec2<f32> {
    let u = 1.0 - t;
    
    let tangent = 2.0 * u * (p1 - p0) + 2.0 * t * (p2 - p1);
    
    return tangent;
}