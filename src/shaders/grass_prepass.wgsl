#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_types::StandardMaterial,
    pbr_bindings,
    pbr_bindings::material,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_world},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
    utils::rand_f,
}
#import bevy_render::maths::PI_2
#import bevy_procedural_grass::grass_types::GrassMaterial;

@group(2) @binding(100)
var<uniform> grass: GrassMaterial;
@group(2) @binding(101)
var texture: texture_2d<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = grass.width * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y); // TODO: change this
    position.x *= width;

    let p0 = vec2<f32>(0.0);
    let p2 = vec2<f32>(1.0, 0.7);
    var curve = grass.curve;
    var midpoint = 0.5;

    let p1 = vec2<f32>(midpoint, curve);
    let bezier = quadratic_bezier(vertex.uv.y, p0, p1, p2);

    position.y = bezier.y;
    position.z = bezier.x;

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));

    position = rotate(position, facing);
    position += ipos;
    
    var out: VertexOutput;
    out.world_position = mesh_position_local_to_world(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.position = position_world_to_clip(out.world_position.xyz);
    out.clip_position_unclamped = out.position;
    out.position.z = min(out.position.z, 1.0);
    out.uv = vertex.uv;

    return out;
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