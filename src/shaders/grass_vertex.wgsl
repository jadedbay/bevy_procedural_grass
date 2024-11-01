#ifdef PREPASS_PIPELINE
    #import bevy_pbr::{
        prepass_io::VertexOutput,
        mesh_functions::{get_world_from_local, mesh_position_local_to_world},
        view_transformations::position_world_to_clip, 
        mesh_view_bindings::view,
    }
    #import bevy_render::globals::Globals,
#else
    #import bevy_pbr::{
        pbr_types::StandardMaterial,
        pbr_bindings, pbr_bindings::material,
        pbr_fragment::pbr_input_from_standard_material,
        pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
        mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
        mesh_view_bindings::{view, globals},
    }
    #import bevy_procedural_grass::VertexOutput
#endif

#import bevy_pbr::utils::rand_f
#import bevy_render::maths::{PI_2, PI}
#import bevy_procedural_grass::{
    GrassMaterial,
    identity_matrix, rotate, quadratic_bezier, bezier_tangent, rotate_x,
    grass_material as grass, grass_texture, wind_texture
};

#ifdef PREPASS_PIPELINE
    @group(0) @binding(1) var<uniform> globals: Globals;

    struct Vertex {
        @builtin(instance_index) instance_index: u32,
        @location(0) position: vec3<f32>,
        @location(1) uv: vec2<f32>,

        @location(3) i_pos: vec4<f32>,
        @location(4) i_chunk_uv: vec2<f32>,
        @location(5) i_facing: vec2<f32>,
        @location(8) i_length: f32,
        @location(9) i_tilt: f32,
        @location(10) i_midpoint: f32,
        @location(11) i_curve: f32,
    }
#else
    struct Vertex {
        @builtin(instance_index) instance_index: u32,
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,

        @location(3) i_pos: vec4<f32>,
        @location(4) i_chunk_uv: vec2<f32>,
        @location(5) i_facing: vec2<f32>,
        @location(6) i_tip_color: vec4<f32>,
        @location(7) i_base_color: vec4<f32>,
        @location(8) i_length: f32,
        @location(9) i_tilt: f32,
        @location(10) i_midpoint: f32,
        @location(11) i_curve: f32,
    };
#endif

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = grass.width * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y); // TODO: change this
    position.x *= width;

    var x_vector: vec2<f32>;
    if (position.x > 0.0) { x_vector = vec2<f32>(-1.0, 0.0); } else { x_vector = vec2<f32>(1.0, 0.0); };

    let t = sample_wind_texture(vertex.i_chunk_uv) * grass.wind_strength; 

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);

    let p0 = vec2<f32>(0.0);
    let angle = vertex.i_tilt;
    var p2 = vec2<f32>(cos(angle), sin(angle)) * vertex.i_length;
    let midpoint = (p2 - p0) * vertex.i_midpoint;
    let blade_normal = normalize(vec2<f32>(-p2.y, p2.x));
    var p1 = midpoint + blade_normal * vertex.i_curve * vertex.i_length;

    let r = rand_f(&state);
    let oscillation = (sin(globals.time * grass.oscillation_speed + (1.0 - vertex.uv.y) * grass.oscillation_flexibility + r * PI_2) * 0.5 + 0.5) * grass.oscillation_strength;
    p1 -= blade_normal * oscillation;
    p2 -= blade_normal * oscillation;

    // let rad = wind.direction * PI / 180.0;
    // let direction = vec2<f32>(cos(rad), sin(rad));

    var bezier = quadratic_bezier(vertex.uv.y, p0, p1, p2);

    position.y = bezier.y;
    position.z = bezier.x;
    #ifndef PREPASS_PIPELINE
        let tangent = normalize(bezier_tangent(vertex.uv.y, p0, p1, p2));
        var normal = normalize(vec3<f32>(0.0, tangent.x, -tangent.y));
        // normal = apply_wind(normal, t);

        // view dependent thickening: sorta works
        let world_view_dir = -normalize(view.view_from_world[2].xyz);
        let local_view_dir = vec3<f32>(
            dot(world_view_dir.xz, vertex.i_facing),
            world_view_dir.y,
            dot(world_view_dir.xz, vec2(-vertex.i_facing.y, vertex.i_facing.x))
        );
        let raw_vd = dot(x_vector, local_view_dir.xz);
        let vd = raw_vd * smoothstep(0.5, 1.0, abs(raw_vd));
        position += normal * vd * width * 0.2;
    #endif

    position = rotate(position, vertex.i_facing);

    // position = rotate(position, mix(vertex.i_facing, grass.wind_direction, grass.wind_strength));

    // position = apply_wind(position, t, vertex.uv.y);
    // #ifndef PREPASS_PIPELINE
    //     normal = apply_wind(normal, t, vertex.uv.y);
    // #endif
    position += ipos;
    
    var out: VertexOutput;
    #ifdef PREPASS_PIPELINE
        out.world_position = mesh_position_local_to_world(
            identity_matrix,
            vec4<f32>(position, 1.0)
        );
        out.position = position_world_to_clip(out.world_position.xyz);
        #ifdef DEPTH_CLAMP_ORTHO
            out.clip_position_unclamped = out.position;
            out.position.z = min(out.position.z, 1.0);
        #endif
    #else
        out.position = mesh_position_local_to_clip(
            identity_matrix,
            vec4<f32>(position, 1.0)
        );
        out.world_position = vec4<f32>(position, 1.0);
        
        out.world_normal = normal;
        out.facing = vertex.i_facing;
        out.tip_color = vertex.i_tip_color;
        out.base_color = vertex.i_base_color;
    #endif

    out.uv = vertex.uv;

    return out;
}

fn sample_wind_texture(uv: vec2<f32>) -> f32 {
    let texture_size = textureDimensions(wind_texture);

    let scrolled_uv = uv + grass.wind_direction * globals.time * 0.2;
    let pixel_coords = vec2<i32>(fract(scrolled_uv) * vec2<f32>(texture_size));
    return textureLoad(wind_texture, pixel_coords, 0).r;
}

fn apply_wind(in: vec3<f32>, t: f32, y: f32) -> vec3<f32> {
    return rotate_around_direction(in, grass.wind_direction, sin(-t) * (y + 0.5));
}

fn rotate_around_direction(v: vec3<f32>, direction: vec2<f32>, angle: f32) -> vec3<f32> {
    // Create a perpendicular direction by rotating -90 degrees (y, -x)
    let perp_direction = vec2<f32>(direction.y, -direction.x);
    
    // Use the perpendicular direction as the rotation axis
    let dir_normalized = normalize(vec3<f32>(perp_direction.x, 0.0, perp_direction.y));
    
    // Rodrigues rotation formula
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    
    return v * cos_angle + 
           cross(dir_normalized, v) * sin_angle + 
           dir_normalized * dot(dir_normalized, v) * (1.0 - cos_angle);
}