#import bevy_pbr::mesh_functions::mesh_position_local_to_clip
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::lights
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::utils::PI
#import bevy_pbr::utils::random1D
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions
#import bevy_pbr::shadows

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
    @location(4) i_normal: vec3<f32>,
    @location(5) i_chunk_uvw: vec3<f32>,
};

struct Color {
    ao: vec4<f32>,
    color_1: vec4<f32>,
    color_2: vec4<f32>,
};
@group(2) @binding(0)
var<uniform> color: Color;

struct Blade {
    length: f32,
    width: f32,
    tilt: f32,
    tilt_variance: f32,
    p1_flexibility: f32,
    p2_flexibility: f32,
    curve: f32,
    specular: f32,
}
@group(2) @binding(1)
var<uniform> blade: Blade;

struct Wind {
    speed: f32,
    amplitude: f32,
    frequency: f32,
    direction: f32,
    oscillation: f32,
    scale: f32,
    _padding: vec2<f32>,
};
@group(3) @binding(0)
var<uniform> wind: Wind;

@group(3) @binding(1)
var t_wind_map: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_position: vec3<f32>,
    @location(4) world_normal: vec3<f32>,
    @location(5) bezier_tangent: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let uv = vertex.uv;

    var hash_id = random1D(vertex.i_pos.x * 100. + vertex.i_pos.y * 100. + vertex.i_pos.z * 0.05 + 2.);
    hash_id = random1D(hash_id * 100000.);

    var position = vertex.position;

    let rad = wind.direction * PI / 180.0;
    let wind_direction = vec2<f32>(cos(rad), sin(rad));
    var facing = normalize(vec2<f32>(mix(-1., 1., hash_id), mix(-1., 1., random1D(hash_id * vertex.i_pos.x))));

    let random_point = vec2<f32>(fract(vertex.i_pos.x * 0.1 * hash_id), fract(vertex.i_pos.y * 0.1 * hash_id));
    let r = sample_wind_map(random_point, wind.speed).r;
    
    var wind_pos = fract(vec2<f32>(vertex.i_pos.x, vertex.i_pos.z) / wind.scale);
    let sample = sample_wind_map(wind_pos, wind.speed).rgb;
    let t = unpack_float(sample);

    let blade_length = mix(blade.length, blade.length + blade.length / 2., fract(hash_id));

    let theta = 2.0 * PI * random1D(hash_id);
    let radius = blade_length * mix(blade.tilt - blade.tilt_variance, blade.tilt, fract(hash_id * 123.));
    var xz = radius * vec2<f32>(cos(theta), sin(theta)); 
    let base_p3 = vec3<f32>(xz.x, sqrt(blade_length * blade_length - dot(xz, xz)), xz.y);
    let base_normal = normalize(vec2<f32>(-base_p3.z, base_p3.x));

    //let xz_displacement = sample_displacement_image(vertex.i_chunk_uvw.xz);

    //let angle = xz_displacement.r * 2.0 * PI;
    //let displace_direction = vec2<f32>(-cos(angle), -sin(angle));
    //var displace_strength = xz_displacement.a * (1.0 - clamp(abs(xz_displacement.b - vertex.i_chunk_uvw.y) / (length / 30.0), 0.0, 1.0));
    
    //xz += displace_direction * (length + blade.tilt) * displace_strength;

    xz += -wind_direction * (0.5 * (sin(t * wind.frequency))) * wind.amplitude;
    xz += base_normal * sin(r * 0.2) * wind.oscillation;

    var y = max(-pow((length(xz) * 0.5), 2.) + blade_length, 0.01);
    var p3 = vec3<f32>(xz.x, y, xz.y);

    let p0 = vec3<f32>(0.0);
    var p1 = (0.33) * p3;
    var p2 = (0.66) * p3;

    var blade_dir_normal = normalize(vec2<f32>(-p3.z, p3.x));
    var blade_normal = normalize(cross(normalize(p3), vec3<f32>(blade_dir_normal.x, 0., blade_dir_normal.y)));

    let distance = distance(base_p3, p3);

    p1 += blade_normal * (y - blade_length) * mix(blade.p1_flexibility, blade.p1_flexibility + 0.2, fract(hash_id * 99.));
    p2 += blade_normal * (y - blade_length) * mix(blade.p2_flexibility, blade.p2_flexibility + 0.2, fract(hash_id * 2480.));

    let bezier = cubic_bezier(uv.y, p0, p1, p2, p3);
    let tangent = bezier_tangent(uv.y, p0, p1, p2, p3);
    position.y = bezier.y;
    let width = blade.width * (1.0 - pow(uv.y, 2.));
    let xz_pos = bezier.xz + (base_normal * vertex.position.x * width);
    position.x = xz_pos.x;
    position.z = xz_pos.y;

    let rotation_matrix = rotate_align(vec3<f32>(0.0, 1.0, 0.0), vertex.i_normal);
    position = rotation_matrix * position;

    var normal = normalize(cross(tangent, vec3<f32>(blade_dir_normal.x, 0.0, blade_dir_normal.y)));
    normal = rotation_matrix * normal;
    out.normal = normal;
    
    position += vertex.i_pos.xyz;

    out.clip_position = mesh_position_local_to_clip(
        identity_matrix, 
        vec4<f32>(position, 1.0)
    );

    out.uv = uv;
    out.world_position = position;
    out.world_normal = vertex.i_normal;
    out.bezier_tangent = tangent;

    return out;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var normal = in.normal;

    let uv_x_transformed = in.uv.x * 2.0 - 1.0;
    var normal_curve = blade.curve * -1.;

    if (!is_front) {
        normal = -normal;
        normal_curve = blade.curve;
    }
    normal = normalize(rotate_vector(normal, in.bezier_tangent, normal_curve * uv_x_transformed));

    let base_color_gradient = mix(color.color_1, color.color_2, in.uv.y);
    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0), in.uv.y);

    let distance = length(view.world_position - in.world_position);
    let spec_strength = mix(0.5, 0.0, clamp((distance - 20.0) / 20.0, 0.0, 1.0)) * blade.specular;

    let view_dir = normalize(view.world_position - in.clip_position.xyz);

    var specular = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var ndotl = 0.0;
    var world_ndotl = 0.0;
    var color_gradient = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var backlight_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let view_z = dot(vec4<f32>(
        view.inverse_view[0].z,
        view.inverse_view[1].z,
        view.inverse_view[2].z,
        view.inverse_view[3].z
    ), vec4<f32>(in.world_position, 1.0));

    let n_directional_lights = lights.n_directional_lights;
    for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
        let light_dir = lights.directional_lights[i].direction_to_light;
        let reflect_dir = reflect(light_dir, in.normal);
        let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.);

        world_ndotl += clamp(dot(in.world_normal, light_dir), 0., 1.);
        ndotl += clamp(dot(normal, light_dir), world_ndotl, 1.0);

        let shadow = clamp(shadows::fetch_directional_shadow(i, vec4<f32>(in.world_position, 1.0), in.world_normal, view_z), 0.1, 1.0);
        if (shadow == 1.0) {
            specular += spec_strength * spec * lights.directional_lights[i].color;
        }

        color_gradient += (base_color_gradient * lights.directional_lights[i].color * shadow) * 0.1;
    }

    let final_color = ((color_gradient + specular) * ndotl * world_ndotl * ao);

    return final_color;
}

fn rotate_vector(v: vec3<f32>, n: vec3<f32>, degrees: f32) -> vec3<f32> {
    let theta = degrees * PI / 180.;
    let cos_theta = cos(theta);
    let sin_theta = sin(theta);

    return v * cos_theta + cross(n, v) * sin_theta + n * dot(n, v) * (1.0 - cos_theta);
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
    
    let scrolled_uv = uv + direction * globals.time * speed;
    
    let pixel_coords = vec2<i32>(fract(scrolled_uv) * vec2<f32>(texture_size));
    return textureLoad(t_wind_map, pixel_coords, 0);
}

const identity_matrix: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

fn unpack_float(rgb: vec3<f32>) -> f32 {
    let r = rgb.r * 255.0;
    let g = rgb.g * 255.0;
    let b = rgb.b * 255.0;

    let noise_scaled = r * 65536.0 + g * 256.0 + b;
    let noise = noise_scaled / 16777215.0;

    return noise;
}