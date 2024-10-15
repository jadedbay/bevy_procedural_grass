#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_types::StandardMaterial,
    pbr_bindings, pbr_bindings::material,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    mesh_view_bindings::{view, globals},
}
#import bevy_pbr::utils::rand_f
#import bevy_render::maths::PI_2
#import bevy_procedural_grass::{
    GrassMaterial, VertexOutput as GrassVertexOutput,
    identity_matrix, rotate, quadratic_bezier, bezier_tangent, rotate_x,
    grass_material as grass, grass_texture,
};

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
        pbr_input.material.base_color * grass.texture_strength, 
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
    // return vec4<f32>(in.t.xyz, 1.0);
}