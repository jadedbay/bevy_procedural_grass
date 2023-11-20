struct GrassData {
    position: vec3<f32>,
    normal: vec3<f32>,
    world_uv: vec2<f32>,
}
@group(0) @binding(0)
var<storage, read_write> data: array<DataAligned>;

struct DataAligned {
    position_x: f32,
    position_y: f32,
    position_z: f32,
    normal_x: f32,
    normal_y: f32,
    normal_z: f32,
    world_uv_x: f32,
    world_uv_y: f32,
}

@compute @workgroup_size(32, 32, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var grass_data: GrassData;
    grass_data.position = vec3<f32>(f32(id.x), 0., f32(id.y));
    grass_data.normal = vec3<f32>(0., 1., 0.);
    grass_data.world_uv = vec2<f32>(0., 0.);

    var data_aligned: DataAligned;
    data_aligned.position_x = grass_data.position.x;
    data_aligned.position_y = grass_data.position.y;
    data_aligned.position_z = grass_data.position.z;
    data_aligned.normal_x = grass_data.normal.x;
    data_aligned.normal_y = grass_data.normal.y;
    data_aligned.normal_z = grass_data.normal.z;
    data_aligned.world_uv_x = grass_data.world_uv.x;
    data_aligned.world_uv_y = grass_data.world_uv.y;

    let index: u32 = id.x + id.y * u32(8);
    data[index] = data_aligned;
}