#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_types::StandardMaterial,
    pbr_bindings,
    pbr_bindings::material,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    mesh_view_bindings::view,
    utils::rand_f,
}
#import bevy_render::maths::PI_2
#import bevy_procedural_grass::grass_types::GrassMaterial;

@group(2) @binding(100)
var<uniform> grass_material: GrassMaterial;
@group(2) @binding(101)
var texture: texture_2d<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
};

struct GrassVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) facing: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> GrassVertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = 0.05 * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y);
    position.x *= width;

    let p0 = vec2<f32>(0.0);
    let p2 = vec2<f32>(1.0, 1.0);
    var curve = grass_material.curve;
    var midpoint = 0.5;

    let p1 = vec2<f32>(midpoint, curve);
    let bezier = quadratic_bezier(vertex.uv.y, p0, p1, p2);
    let tangent = normalize(bezier_tangent(vertex.uv.y, p0, p1, p2));

    var normal = normalize(vec3<f32>(0.0, tangent.x, -tangent.y));

    position.y = bezier.y;
    position.z = bezier.x;

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));

    position = rotate(position, facing);
    position += ipos;
    
    var out: GrassVertexOutput;
    out.position = mesh_position_local_to_clip(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.world_position = vec4<f32>(position, 1.0);
    
    out.world_normal = normal;
    out.facing = facing;

    out.uv = vertex.uv;

    return out;
}

@fragment fn fragment(
    in: GrassVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let midrib_softness = 0.08;
    let rim_position = 0.5;
    let rim_softness = 0.08;
    let normal_strength = 0.3;

    let uv_mid = in.uv.x - 0.5;
    let midrib = smoothstep(-midrib_softness, midrib_softness, uv_mid);
    let rim = smoothstep(rim_position, rim_position - rim_softness, abs(uv_mid));
    let blend = mix(1.0 - midrib, midrib, rim);
    let normal_x = normal_strength * mix(1.0, -1.0, blend);

    var vo: VertexOutput;
    vo.position = in.position;
    vo.world_position = in.world_position;
    vo.world_normal = in.world_normal;
    vo.world_normal.x = normal_x;
    vo.uv = in.uv;

    vo.world_normal = rotate(vo.world_normal, in.facing);

    var pbr_input = pbr_input_from_standard_material(vo, is_front);

    let sampled_texture = textureSampleBias(texture, pbr_bindings::base_color_sampler, in.uv, view.mip_bias);
    pbr_input.material.base_color = mix(pbr_input.material.base_color * 0.65, pbr_input.material.base_color, sampled_texture);
    let roughness = 0.6;
    let roughness_variance = 0.15;
    pbr_input.material.perceptual_roughness = mix(roughness - roughness_variance, roughness, sampled_texture.r);
    let reflectance = 0.1;
    let reflectance_variance = 0.1;
    pbr_input.material.reflectance = mix(reflectance - reflectance_variance, reflectance, sampled_texture.r);

    var color = apply_pbr_lighting(pbr_input);
    color = main_pass_post_lighting_processing(pbr_input, color);
    return color;

    // return vec4<f32>(pbr_input.world_normal, 1.0);
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

fn rotate_around_axis(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    return v * cos_angle + cross(axis, v) * sin_angle + axis * dot(axis, v) * (1.0 - cos_angle);
}