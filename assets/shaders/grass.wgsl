#import bevy_pbr::mesh_functions  mesh_position_local_to_clip
#import bevy_pbr::mesh_bindings   mesh

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
    @location(4) i_uv: vec2<f32>
};

struct Color {
    ao: vec4<f32>,
    color_1: vec4<f32>,
    color_2: vec4<f32>,
    tip: vec4<f32>,
};
@group(2) @binding(0)
var<uniform> color: Color;

struct Wind {
    frequency: f32,
    speed: f32,
    noise: f32,
    strength: f32,
    time: f32,
};
@group(3) @binding(0)
var<uniform> wind: Wind;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_uv: vec2<f32>,
    @location(3) time: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let uv = 1. - vertex.uv;

    var hash_id = hash(vertex.i_pos.x * 10000. + vertex.i_pos.y * 100. + vertex.i_pos.z * 0.05 + 2.);
    hash_id = hash(hash_id * 100000.);
    let fract_id = fract(hash_id);

    var position = vertex.position;

    let noise_value = noise(vertex.i_uv.x + vertex.i_uv.y) * wind.noise;
    let t = sin(wind.frequency * ((-wind.time * wind.speed) + vertex.i_uv.x + vertex.i_uv.y + noise_value)); 
    
    let curve_variance = mix(1.5, 2.0, fract_id);
    position.z += mix(uv.y * uv.y, uv.y * uv.y * curve_variance, fract_id);

    position = rotate_y(position, hash_id * 180.); // rotation

    let wind_variance = wind.strength * mix(1.5, 2.0, fract_id);
    position.z += mix(uv.y * uv.y, uv.y * uv.y * wind_variance, t);

    let y_scale = mix(0., 0.4, fract_id); // height
    position.y *= 1. + y_scale;
    position.y += y_scale;
    
    position += vertex.i_pos.xyz;

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        mesh.model, 
        vec4<f32>(position, 1.0)
    );

    out.uv = uv;
    out.world_uv = vertex.i_uv;
    out.time = t;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color_gradient = mix(color.color_1, color.color_2, in.uv.y);

    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0),  in.uv.y);
    let tip = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.tip,  in.uv.y * in.uv.y);

    let final_color = (color_gradient + tip) * ao;
    
    return final_color;
    //return vec4(in.time, in.time, in.time, 1.);
}

const PI: f32 = 3.141592653589793238;

fn rotate_y(vertex: vec3<f32>, degrees: f32) -> vec3<f32> {
    let alpha: f32 = degrees * PI / 180.0;
    let sina: f32 = sin(alpha);
    let cosa: f32 = cos(alpha);
    let m: mat2x2<f32> = mat2x2<f32>(cosa, -sina, sina, cosa);
    let rotated_xz: vec2<f32> = m * vertex.xz;
    return vec3<f32>(rotated_xz.x, vertex.y, rotated_xz.y);
}

fn hash(n: f32) -> f32 {
    let x = fract(n * 0.1031);
    return x * x * 33.33 + x;
}

fn noise(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453);
}