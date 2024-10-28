#import bevy_pbr::utils::rand_f
#import bevy_render::maths::{PI, PI_2}
#import bevy_procedural_grass::{GrassInstance, Aabb2d, GrassMaterial, GrassClump};

const INFINITY = 3.402823e+38;

@group(0) @binding(0) var<storage, read_write> output: array<GrassInstance>;
@group(0) @binding(1) var heightmap: texture_2d<f32>;
@group(0) @binding(2) var<uniform> height_scale: f32;
@group(0) @binding(3) var<uniform> height_offset: f32;
@group(0) @binding(4) var<uniform> chunk_aabb: Aabb2d;
@group(0) @binding(5) var<uniform> aabb: Aabb2d;

@group(1) @binding(0) var<uniform> clump_aabb: Aabb2d;
@group(1) @binding(1) var<uniform> clump_size: vec2<f32>;
@group(1) @binding(2) var<storage, read> clump_positions: array<vec2<f32>>;
@group(1) @binding(3) var<storage, read> clump_params: array<GrassClump>;

#import bevy_procedural_grass::grass_material as grass

@compute @workgroup_size(512)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>, 
    @builtin(local_invocation_id) local_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    var state: u32 = global_id.x + u32(chunk_aabb.min.x) * 1000u + u32(chunk_aabb.min.y) * 2000u;
    let u = rand_f(&state);
    state = state * 747796405u + 2891336453u;
    let v = rand_f(&state);
    let local_uv = vec2<f32>(u, v);

    let chunk_position = chunk_aabb.min + (local_uv * (chunk_aabb.max - chunk_aabb.min));
    
    let global_uv = (chunk_position - aabb.min) / (aabb.max - aabb.min); 

    let dimensions = textureDimensions(heightmap);
    var texture_coords = vec2<i32>(global_uv * vec2<f32>(dimensions));

    texture_coords = max(vec2<i32>(0), min(texture_coords, vec2<i32>(dimensions) - vec2<i32>(1)));

    let height = textureLoad(heightmap, texture_coords, 0).r;

    var instance: GrassInstance;
    instance.position = vec4<f32>(chunk_position.x, height * height_scale + height_offset, chunk_position.y, 1.0);
    instance.chunk_uv = local_uv;

    let clump_cell = vec2<u32>(
        u32(floor((instance.position.x - clump_aabb.min.x) / f32(clump_size.x))),
        u32(floor((instance.position.z - clump_aabb.min.y) / f32(clump_size.y)))
    );

    let clump_count = vec2<u32>(
        u32(ceil((clump_aabb.max.x - clump_aabb.min.x) / f32(clump_size.x))),
        u32(ceil((clump_aabb.max.y - clump_aabb.min.y) / f32(clump_size.y)))
    ); 

    var closest_distance: f32 = INFINITY;
    var clump_index: u32 = 0u;
    var clump_position: vec2<f32>;
    for (var dx: i32 = -1; dx <= 1; dx++) {
        for (var dy: i32 = -1; dy <= 1; dy++) {
            let neighbor_x = i32(clump_cell.x) + dx;
            let neighbor_y = i32(clump_cell.y) + dy;
            
            if (neighbor_x >= 0 && neighbor_x < i32(clump_count.x) && neighbor_y >= 0 && neighbor_y < i32(clump_count.y)) {
                let neighbor_index = u32(neighbor_x) * clump_count.y + u32(neighbor_y);
                let clump_pos = clump_positions[neighbor_index];
                
                let distance = distance(vec2<f32>(instance.position.x, instance.position.z), clump_pos);
                
                if (distance < closest_distance) {
                    closest_distance = distance;
                    clump_index = neighbor_index;
                    clump_position = clump_pos;
                }
            }
        }
    }

    let clump_facing = clump_params[clump_index].facing;
    if (clump_facing.x == 2.0) {
        let direction = vec2<f32>(
            instance.position.x - clump_position.x,
            -(instance.position.z - clump_position.y)
        );
        instance.facing = normalize(vec2<f32>(direction.y, -direction.x));        
    } else if (clump_facing.x == 3.0) {
        let direction = vec2<f32>(
            instance.position.x - clump_position.x,
            -(instance.position.z - clump_position.y)
        );
        let random_angle = (rand_f(&state) - 0.5) * 2.0;
        let rotation_matrix = mat2x2<f32>(
            cos(random_angle), -sin(random_angle),
            sin(random_angle), cos(random_angle)
        );
        let base_facing = normalize(vec2<f32>(-direction.y, direction.x));
        instance.facing = rotation_matrix * base_facing;  
    } else if (clump_facing.x == 4.0) {
        let facing_angle: f32 = rand_f(&state) * PI_2;
        let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));
        instance.facing = facing;       
    } else {
        let random_angle = (rand_f(&state) - 0.5) * 2.0; 
        let base_facing = clump_params[clump_index].facing;
        let rotation_matrix = mat2x2<f32>(
            cos(random_angle), -sin(random_angle),
            sin(random_angle), cos(random_angle)
        );
        instance.facing = rotation_matrix * base_facing;
    }


    var param_state: u32 = u32(instance.position.x * 500);
    let length = clump_params[clump_index].length * grass.length;
    instance.length = mix(length - 0.2, length + 0.2, rand_f(&param_state));
    param_state = u32(instance.position.y * 9000);
    instance.tilt = mix(grass.tilt - 0.2, grass.tilt + 0.2, rand_f(&state));
    param_state = u32(instance.facing.x * 100);
    instance.midpoint = mix(grass.midpoint - 0.2, grass.midpoint + 0.2, rand_f(&state));
    param_state = u32(instance.position.z * 200);
    instance.curve = mix(grass.curve - 0.2, grass.curve + 0.2, rand_f(&state));
    
    output[global_id.x] = instance;
}
