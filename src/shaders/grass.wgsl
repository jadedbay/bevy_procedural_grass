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
#import bevy_procedural_grass::{
    GrassMaterial,
    identity_matrix, rotate, quadratic_bezier, bezier_tangent,
    grass_material as grass, grass_texture,
};

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

    let width = grass.width * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y); // TODO: change this
    position.x *= width;

    let p0 = vec2<f32>(0.0);
    let p2 = vec2<f32>(1.0, 0.7);
    var curve = grass.curve;
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

@fragment 
fn fragment(
    in: GrassVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {

    // TODO: move this to a compute shader and use 1d texture
    let uv_mid = in.uv.x - 0.5;
    let midrib = smoothstep(-grass.midrib_softness, grass.midrib_softness, uv_mid);
    let rim = smoothstep(grass.rim_position, grass.rim_position - grass.rim_softness, abs(uv_mid));
    let blend = mix(1.0 - midrib, midrib, rim);
    let normal_x = grass.width_normal_strength * mix(1.0, -1.0, blend);

    var vo: VertexOutput;
    vo.position = in.position;
    vo.world_position = in.world_position;
    vo.world_normal = in.world_normal;
    vo.world_normal.x = normal_x;
    vo.uv = in.uv;

    vo.world_normal = rotate(vo.world_normal, in.facing);

    var pbr_input = pbr_input_from_standard_material(vo, is_front);

    let sampled_texture = textureSampleBias(grass_texture, pbr_bindings::base_color_sampler, in.uv, view.mip_bias);

    let ao = mix(grass.min_ao, 1.0, in.uv.y);
    pbr_input.material.base_color = mix(
        pbr_input.material.base_color * 0.65, 
        pbr_input.material.base_color, 
        sampled_texture
    ) * ao;
    pbr_input.material.perceptual_roughness = mix(
        material.perceptual_roughness - grass.roughness_variance, 
        material.perceptual_roughness, 
        sampled_texture.r
    );
    pbr_input.material.reflectance = mix(
        material.reflectance - grass.reflectance_variance, 
        material.reflectance, 
        sampled_texture.r
    );

    var color = apply_pbr_lighting(pbr_input);
    color = main_pass_post_lighting_processing(pbr_input, color);

    return color;

    // return vec4<f32>(pbr_input.world_normal, 1.0);
}