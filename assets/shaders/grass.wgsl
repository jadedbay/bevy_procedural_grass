#import bevy_pbr::mesh_functions  mesh_position_local_to_clip
#import bevy_pbr::mesh_bindings   mesh

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
    @location(4) i_normal: vec3<f32>,
    @location(5) i_uv: vec2<f32>
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

struct Light {
    direction: vec3<f32>,
}
@group(4) @binding(0)
var<uniform> light: Light;

// struct Camera {
    
// }

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_uv: vec2<f32>,
    @location(4) t: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let uv = vertex.uv;

    var hash_id = hash(vertex.i_pos.x * 10000. + vertex.i_pos.y * 100. + vertex.i_pos.z * 0.05 + 2.);
    hash_id = hash(hash_id * 100000.);
    let fract_id = fract(hash_id);

    var position = vertex.position;

    let noise_value = noise(vertex.i_uv.x + vertex.i_uv.y) * wind.noise;
    let t = sin(wind.frequency * ((-wind.time * wind.speed) + vertex.i_uv.x + vertex.i_uv.y + noise_value)); 

    let length = mix(1.5, 2.5, fract(hash(hash_id)));
    let width = mix(1.5, 2.5, fract(hash(hash_id * 200. + 5.)));

    let tilt = mix(0.8, length, fract(hash(hash_id * 10000.)));
    let curve = mix(0.66, 1., fract(hash(hash_id * 8000. + 50.)));

    let sway = mix(0.5, 0.8, fract_id);
    let p2 = mix(0.0, sway, t * uv.y);

    let bezier = cubic_bezier(uv.y * 0.8, vec2(0.0, 0.0), vec2(-curve + 1.33, curve), vec2(length, tilt - p2), vec2(length, tilt));
    position.z = bezier.x;
    position.y = bezier.y;
    position.x *= width;

    position = rotate_y(position, hash_id * 3.);

    let rotation_matrix = rotate_align(vec3<f32>(0.0, 1.0, 0.0), vertex.i_normal);
    position = rotation_matrix * position;

    position += vertex.i_pos.xyz;

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        mesh.model, 
        vec4<f32>(position, 1.0)
    );

    out.uv = uv;
    out.world_uv = vertex.i_uv;
    out.t = t;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // var normal = in.normal;
    // let view_dir = normalize(camera_position - in.world_pos);
    // if (dot(view_dir, normal) < 0.0) {
    //     normal = -normal;
    // }

    let color_gradient = mix(color.color_1, color.color_2, in.uv.y);
    //let ndotl = clamp(dot(light.direction, in.normal), 0.0, 1.0);

    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0),  in.uv.y);
    let tip = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.tip,  in.uv.y * in.uv.y);

    let final_color = (color_gradient + tip) * ao;
    
    return final_color;
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

fn cubic_bezier(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec2<f32> {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    var p = uuu * p0; // (1-t) ^ 3  * p0
    p = p + 3.0 * uu * t * p1; // 3(1-t)^2 * t * p1
    p = p + 3.0 * u * tt * p2; // 3(1-t) * t^2 * p2
    p = p + ttt * p3; // t^3 * p3

    return p;
}

fn rotate_align(v1: vec3<f32>, v2: vec3<f32>) -> mat3x3<f32> {
    let axis = cross(v1, v2);

    let cos_a = dot(v1, v2);
    let k = 1.0 / (1.0 + cos_a);

    let result = mat3x3f( 
            (axis.x * axis.x * k) + cos_a, (axis.x * axis.y * k) + axis.z, (axis.x * axis.z * k) - axis.y,
            (axis.y * axis.x * k) - axis.z, (axis.y * axis.y * k) + cos_a,  (axis.y * axis.z * k) + axis.x, 
            (axis.z * axis.x * k) + axis.y, (axis.z * axis.y * k) - axis.x, (axis.z * axis.z * k) + cos_a 
        );

    return result;
}