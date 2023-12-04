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
    variance: f32,
    direction: f32,
    force: f32,
};
@group(3) @binding(0)
var<uniform> wind: Wind;

struct Blade {
    length: f32,
    width: f32,
    tilt: f32,
    tilt_variance: f32,
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
    var facing = normalize(vec2<f32>(mix(-1., 1., random1D(fract(hash_id) * 5000.)), mix(-1., 1., random1D(hash_id * vertex.i_pos.x))));

    let random_point = vec2<f32>(fract(vertex.i_pos.x * 0.1 * hash_id), fract(vertex.i_pos.y * 0.1 * hash_id));
    let r = sample_wind_map(random_point, wind.speed).r;
    
    var t = sample_wind_map(vertex.i_uv, wind.speed).r;

    let width = blade.width;
    let length = mix(blade.length, blade.length + 0.6, fract(hash_id));

    let theta = 2.0 * PI * random1D(hash_id);
    let radius = length * mix(blade.tilt_variance, blade.tilt, fract(random1D(hash_id)));
    var xz = radius * vec2<f32>(cos(theta), sin(theta)); 
    let base_xz = xz;

    xz += -wind_direction * (sin(mix(wind.strength - wind.variance, wind.strength, t) * wind.speed) * wind.force);
    let xz_length = length(xz);
    let clamped_length = clamp(xz_length, 0.0, length - 0.05);
    xz = normalize(xz) * clamped_length;

    var y = sqrt(length * length - dot(xz, xz));
    //y += (xz_length - clamped_length) * wind.strength * length;
    var p3 = vec3<f32>(xz.x, y, xz.y);

    let p0 = vec3<f32>(0.0);

    var p1 = 0.33 * p3;
    var p2 = 0.66 * p3;

    let blade_dir_normal = normalize(vec2<f32>(-xz.y, xz.x));
    let blade_normal = normalize(cross(vec3<f32>(blade_dir_normal.x, 0., blade_dir_normal.y), p3));
    
    p1 += blade_normal * (length - y) * blade.bend;
    p2 += blade_normal * (length - y) * blade.bend;
    p2 -= blade_normal * -sin(r * 0.2);
    p3 += blade_normal * sin(r * 0.2) * 1.5;

    let bezier = cubic_bezier(uv.y, p0, p1, p2, p3);
    let tangent = bezier_tangent(uv.y, p0, p1, p2, p3);
    position.y = bezier.y;
    let xz_pos = bezier.xz + (normalize(vec2<f32>(-base_xz.y, base_xz.x)) * vertex.position.x * width);
    position.x = xz_pos.x;
    position.z = xz_pos.y;

    let rotation_matrix = rotate_align(vec3<f32>(0.0, 1.0, 0.0), vertex.i_normal);
    position = rotation_matrix * position;

    var normal = normalize(cross(tangent, vec3<f32>(blade_dir_normal.x, 0.0, blade_dir_normal.y)));
    normal = rotation_matrix * normal;
    out.normal = normal;

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

    let distance = length(view.world_position - in.world_position);
    let spec_strength = mix(0.5, 0.0, clamp((distance - 20.0) / 20.0, 0.0, 1.0));
    

    let view_dir = normalize(view.world_position - in.clip_position.xyz);
    let reflect_dir = reflect(lights.directional_lights[0].direction_to_light, in.normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.);
    let specular =  spec_strength * spec;

    let color_gradient = mix(color.color_1, color.color_2, in.uv.y);
    let ndotl = clamp(dot(normal, lights.directional_lights[0].direction_to_light), 0.3, 1.0);
    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0),  in.uv.y);
    let tip = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.tip,  in.uv.y * in.uv.y);

    let world_ndotl = clamp(dot(in.world_normal, lights.directional_lights[0].direction_to_light), 0., 1.);

    let final_color = (color_gradient + specular) * ndotl * ao * world_ndotl;
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

    var p = uuu * p0;
    p = p + 3.0 * uu * t * p1;
    p = p + 3.0 * u * tt * p2;
    p = p + ttt * p3;

    return p;
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
    
    let scrolled_uv = uv + direction * globals.time * speed * 1.3;
    
    let pixel_coords = vec2<i32>(fract(scrolled_uv) * vec2<f32>(texture_size));
    return textureLoad(t_wind_map, pixel_coords, 0);
}