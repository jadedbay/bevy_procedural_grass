#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::lights
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::utils::PI
#import bevy_pbr::utils::random1D

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
    @location(4) i_normal: vec3<f32>,
    @location(5) i_uv: vec2<f32>,
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
    speed: f32,
    strength: f32,
    direction: f32,
    force: f32,
};
@group(3) @binding(0)
var<uniform> wind: Wind;

struct Blade {
    length: f32,
    width: f32,
    tilt: f32,
    bend: f32,
}
@group(4) @binding(0)
var<uniform> blade: Blade;

@group(5) @binding(0)
var t_wind_map: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_uv: vec2<f32>,
    @location(3) t: f32,
    @location(4) normal: vec3<f32>,
    @location(5) world_position: vec3<f32>,
    @location(6) world_normal: vec3<f32>,
    @location(7) curved_normal: vec3<f32>,
    @location(8) bezier_tangent: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let uv = vertex.uv;

    var hash_id = random1D(vertex.i_pos.x * 10000. + vertex.i_pos.y * 100. + vertex.i_pos.z * 0.05 + 2.);
    hash_id = random1D(hash_id * 100000.);

    var position = vertex.position;

    let rad = wind.direction * PI / 180.0;
    let wind_direction = vec2<f32>(cos(rad), sin(rad));
    var facing = normalize(vec2<f32>(mix(-1., 1., hash_id), mix(-1., 1., random1D(hash_id * vertex.i_pos.x))));

    let noise_value = noise(vertex.i_uv.x + vertex.i_uv.y);
    let r = sample_wind_map(vertex.i_uv * hash_id, 0.1).r;

    //var t = remap(sample_wind_map(vertex.i_uv, wind.speed).r, 0.20, 0.8, 0.0, 1.0);
    var t = sample_wind_map(vertex.i_uv, wind.speed).r;

    let width = blade.width;
    let length = mix(blade.length, blade.length + 0.6, fract(hash_id));
    let tilt = mix(blade.tilt, blade.tilt + 0.2, fract(hash_id * 5000. + 2.));
    let bend = mix(blade.bend, blade.bend + 0.2, fract(hash_id * 200. + 4.));

    let p3y = tilt * length;
    let p3x = sqrt(length * length - p3y * p3y);
    
    var p3 = vec3<f32>(p3x, p3y, 0.0);

    var blade_dir = normalize(vec3<f32>(-p3y, p3x, 0.0));

    var p0 = vec3<f32>(0.0, 0.0,  0.0);
    var p1 = 0.33 * p3;
    var p2 = 0.66 * p3;

    p1 += blade_dir * bend;
    p2 += blade_dir * bend;

    var p2_offset = pow(0.66, wind.strength) * sin(r * 0.5) * wind.force;
    var p3_offset = pow(1., wind.strength) * sin(r * 0.5) * wind.force;

    p2 += blade_dir * p2_offset;
    p3 += blade_dir * p3_offset;

    facing = normalize(mix(facing, wind_direction, sin(t)));

    let bezier = cubic_bezier(uv.y, p0, p1, p2, p3);
    position.y = bezier.y;
    var facing_normal = vec2<f32>(-facing.y, facing.x);
    let bezier_xz = bezier.x * facing;
    let xz_pos = bezier_xz + (facing_normal * vertex.position.x * width);
    position.x = xz_pos.x;
    position.z = xz_pos.y;

    let rotation_matrix = rotate_align(vec3<f32>(0.0, 1.0, 0.0), vertex.i_normal);
    position = rotation_matrix * position;

    let tangent = bezier_tangent(uv.y, p0, p1, p2, p3);
    var normal = normalize(cross(tangent, vec3<f32>(facing_normal.x, 0.0, facing_normal.y)));
    out.normal = normal;


    // normal.x += pow(uv.x * uv.x, 0.5);
    // normal.z += pow(uv.x * uv.x, 0.5);
    //normal = normalize(normal);

    position += vertex.i_pos.xyz;

    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(0u), 
        vec4<f32>(position, 1.0)
    );

    out.uv = uv;
    out.world_uv = vertex.i_uv;
    out.t = r;
    out.curved_normal = normal;
    out.world_position = position;
    out.world_normal = vertex.i_normal;
    out.bezier_tangent = tangent;

    return out;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var normal = in.curved_normal;

    let uv_x_transformed = in.uv.x * 2.0 - 1.0;
    let normal_curve = 15. * -1.;
    normal = normalize(rotate_vector(normal, in.bezier_tangent, normal_curve * uv_x_transformed));
    
    if (!is_front) {
        normal = -normal;
    }

    let spec_strength = 1.;
    let view_dir = normalize(view.world_position - in.clip_position.xyz);
    let reflect_dir = reflect(lights.directional_lights[0].direction_to_light, in.normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.);
    let specular =  spec_strength * spec;

    let color_gradient = mix(color.color_1, color.color_2, in.uv.y);
    let ndotl = clamp(dot(normal, -lights.directional_lights[0].direction_to_light), 0.3, 1.0);
    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0),  in.uv.y);
    let tip = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.tip,  in.uv.y * in.uv.y);

    let final_color = (color_gradient + specular) * ndotl * ao;
    //let final_color = color.color_2 * ndotl;

    return final_color;
    //return vec4(in.t, in.t, in.t, 1.0);
}

fn rotate_y(vertex: vec3<f32>, degrees: f32) -> vec3<f32> {
    let alpha: f32 = degrees * PI / 180.0;
    let sina: f32 = sin(alpha);
    let cosa: f32 = cos(alpha);
    let m: mat2x2<f32> = mat2x2<f32>(cosa, -sina, sina, cosa);
    let rotated_xz: vec2<f32> = m * vertex.xz;
    return vec3<f32>(rotated_xz.x, vertex.y, rotated_xz.y);
}

fn rotate_vector(v: vec3<f32>, n: vec3<f32>, degrees: f32) -> vec3<f32> {
    let theta = degrees * PI / 180.;
    let cosTheta = cos(theta);
    let sinTheta = sin(theta);

    return v * cosTheta + cross(n, v) * sinTheta + n * dot(n, v) * (1.0 - cosTheta);
}

fn cubic_bezier(t: f32, p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>) -> vec3<f32> {
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

fn cubic_bezier_v2(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec2<f32> {
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

fn bezier_derivative(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec2<f32> {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    var p = 3.0 * uu * (p1 - p0); // 3(1-t)^2 * (p1 - p0)
    p = p + 6.0 * u * t * (p2 - p1); // 6(1-t) * t * (p2 - p1)
    p = p + 3.0 * tt * (p3 - p2); // 3t^2 * (p3 - p2)

    return p;
}

fn bezier_tangent_v2(t: f32, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> vec2<f32> {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    
    let tangent = -3.0 * u2 * p0
        + 3.0 * u2 * p1
        - 6.0 * u * t * p1
        + 6.0 * u * t * p2
        - 3.0 * t2 * p2
        + 3.0 * t2 * p3;
    
    return tangent;
}

fn bezier_tangent(t: f32, p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>) -> vec3<f32> {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    
    let tangent = -3.0 * u2 * p0
        + 3.0 * u2 * p1
        - 6.0 * u * t * p1
        + 6.0 * u * t * p2
        - 3.0 * t2 * p2
        + 3.0 * t2 * p3;
    
    return tangent;
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

fn sample_wind_map(uv: vec2<f32>, speed: f32) -> vec4<f32> {
    let texture_size = textureDimensions(t_wind_map);
    
    let rad = wind.direction * PI / 180.0;
    let direction = vec2<f32>(cos(rad), sin(rad));
    
    let scrolled_uv = uv + direction * globals.time * speed;
    
    let pixel_coords = vec2<i32>(fract(scrolled_uv) * vec2<f32>(texture_size));
    return textureLoad(t_wind_map, pixel_coords, 0);
}

fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    return (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min;
}

fn noise(x: f32) -> f32 {
    return fract(sin(x) * 43758.5453);
}
