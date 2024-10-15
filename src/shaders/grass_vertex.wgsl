#ifdef PREPASS_PIPELINE
    #import bevy_pbr::{
        prepass_io::VertexOutput,
        mesh_functions::{get_world_from_local, mesh_position_local_to_world},
        view_transformations::position_world_to_clip,
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
#import bevy_render::maths::PI_2
#import bevy_procedural_grass::{
    GrassMaterial,
    identity_matrix, rotate, quadratic_bezier, bezier_tangent, rotate_x,
    grass_material as grass, grass_texture, wind_texture
};

#ifdef PREPASS_PIPELINE
    @group(0) @binding(1) var<uniform> globals: Globals;

    struct Vertex {
        @location(0) position: vec3<f32>,
        @location(1) uv: vec2<f32>,

        @location(3) i_pos: vec4<f32>,
        @location(4) i_chunk_uv: vec2<f32>,
        @location(5) i_facing: vec2<f32>,
    }
#else
    struct Vertex {
        @location(0) position: vec3<f32>,
        @location(1) normal: vec3<f32>,
        @location(2) uv: vec2<f32>,

        @location(3) i_pos: vec4<f32>,
        @location(4) i_chunk_uv: vec2<f32>,
        @location(5) i_facing: vec2<f32>,
    };
#endif

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = grass.width * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y); // TODO: change this
    position.x *= width;

    let p0 = vec2<f32>(0.0);
    let p2 = vec2<f32>(1.0, 0.0);
    var curve = grass.curve;
    var midpoint = grass.midpoint;

    let t = sample_wind_texture(vertex.i_chunk_uv, 0.0); 
    // curve -= sample_wind_texture(vertex.i_chunk_uv, 0.0);

    let p1 = vec2<f32>(midpoint, curve);
    let bezier = quadratic_bezier(vertex.uv.y, p0, p1, p2);
    let tangent = normalize(bezier_tangent(vertex.uv.y, p0, p1, p2));

    #ifndef PREPASS_PIPELINE
        var normal = normalize(vec3<f32>(0.0, tangent.x, -tangent.y));
        normal = rotate_x(normal, t + 0.3);
    #endif

    position.y = bezier.y;
    position.z = bezier.x;
    position = rotate_x(position, t + 0.3);  


    position = rotate(position, vertex.i_facing);
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
        out.t = vec4<f32>(t);
    #endif

    out.uv = vertex.uv;

    return out;
}

fn sample_wind_texture(uv: vec2<f32>, offset: f32) -> f32 {
    let texture_size = textureDimensions(wind_texture);

    let scrolled_uv = uv + globals.time * 0.2;
    let pixel_coords = vec2<i32>(fract(scrolled_uv + offset) * vec2<f32>(texture_size));
    return textureLoad(wind_texture, pixel_coords, 0).r;
}