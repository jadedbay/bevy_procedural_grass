#import bevy_procedural_grass::grass_types::Aabb;

struct Counts {
    instance_count: u32,
    workgroup_count: atomic<u32>,
    scan_workgroup_count: u32,
    scan_groups_workgroup_count: u32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> areas: array<u32>;

var<push_constant> triangle_count: u32;
@group(1) @binding(0) var<uniform> aabb: Aabb;
@group(1) @binding(1) var<storage, read_write> triangle_dispatch_count: array<u32>;
@group(1) @binding(2) var<storage, read_write> counts: Counts;

@compute @workgroup_size(#{CHUNK_SIZE_X}, #{CHUNK_SIZE_Y}, #{CHUNK_SIZE_Z})
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let base_index = global_id * 3;
    let v0 = positions[base_index].xyz;
    let v1 = positions[base_index + 1].xyz;
    let v2 = positions[base_index + 2].xyz;
    
    let density = 0.5;
    let area = length(cross(v1 - v0, v2 - v0)) / 2.0;
    let blade_count = ceil(density * area);
    
    let dispatch_count = u32(ceil(blade_count / 32.0));

    if (triangle_intersects_aabb(v0, v1, v2)) {
        triangle_dispatch_count[(local_id.z * #{CHUNK_SIZE_X} * #{CHUNK_SIZE_Y} + local_id.y * #{CHUNK_SIZE_X} + local_id.x) * triangle_count + workgroup_id.x] = dispatch_count;
    }     

    atmoicAdd(&counts.workgroup_count, dispatch_count); 
}

fn aabb_contains_point(point: vec3<f32>) -> bool {
  return point.x >= aabb.min.x && point.x <= aabb.max.x &&
  point.y >= aabb.min.y && point.y <= aabb.max.y &&
  point.z >= aabb.min.z && point.z <= aabb.max.z;
}

fn triangle_intersects_aabb(v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> bool {
  if aabb_contains_point(v0) || aabb_contains_point(v1) || aabb_contains_point(v2) {
    return true;
  }

  let f0 = v1 - v0;
  let f1 = v2 - v1;
  let f2 = v0 - v2;

  let axes = array<vec3<f32>, 9>(
    vec3<f32>(0.0, -f0.z, f0.y),
    vec3<f32>(0.0, -f1.z, f1.y),
    vec3<f32>(0.0, -f2.z, f2.y),
    vec3<f32>(f0.z, 0.0, -f0.x),
    vec3<f32>(f1.z, 0.0, -f1.x),
    vec3<f32>(f2.z, 0.0, -f2.x),
    vec3<f32>(-f0.z, f0.x, 0.0),
    vec3<f32>(-f1.z, f1.x, 0.0),
    vec3<f32>(-f2.z, f2.x, 0.0),
  );

  for (var i: i32 = 0; i < 9; i++) {
     if (!overlap_on_axis(v0, v1, v2, axes[i])) {
      return false;
    }
  }

  let axes = array<vec3<f32>, 3>(
    vec3<f32>(1., 0., 0.),
    vec3<f32>(0., 1., 0.),
    vec3<f32>(0., 0., 1.)
  );

  for (var i: i32 = 0; i < 3; i++) {
    if (!overlap_on_axis(v0, v1, v2, axes[i])) {
      return false;
    }
  }

  let normal = cross(f0, f1);
  if (!overlap_on_axis(v0, v1, v2, normal)) {
    return false;
  }

  return true;
}

fn overlap_on_axis(v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>, axis: vec3<f32>) -> bool {
  let t = project_triangle(v0, v1, v2, axis);
  let b = project_aabb(axis);

  return t.y >= b.x && b.y >= t.x;
}

fn project_triangle(v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>, axis: vec3<f32>) -> vec2<f32> {
  let p0 = dot(v0, axis);
  let p1 = dot(v1, axis);
  let p2 = dot(v2, axis);

  return vec2<f32>(min(p0, min(p1, p2)), max(p0, max(p1, p2)));
}

fn project_aabb(axis: vec3<f32>) -> vec2<f32> {
  let vertices = array<vec3<f32>, 8>(
    aabb.min,
    vec3<f32>(aabb.min.x, aabb.min.y, aabb.max.z),
    vec3<f32>(aabb.min.x, aabb.max.y, aabb.min.z),
    vec3<f32>(aabb.min.x, aabb.max.y, aabb.max.z),
    vec3<f32>(aabb.max.x, aabb.min.y, aabb.min.z),
    vec3<f32>(aabb.max.x, aabb.min.y, aabb.max.z),
    vec3<f32>(aabb.max.x, aabb.max.y, aabb.min.z),
    aabb.max,
  );

  var min = dot(vertices[0], axis);
  var max = min;

  for (var i: i32 = 1; i < 8, i++) {
    let projection = dot(vertices[i], axis);
    if (projection < min) {
      min = projection;
    }
    if projection > max {
      max = projection;
    }
  }
  return vec2<f32>(min, max);
}
