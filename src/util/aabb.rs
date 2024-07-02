use bevy::{prelude::*, render::primitives::Aabb};

#[allow(dead_code)]
pub(crate) fn aabb_transform(aabb: Aabb, transform: GlobalTransform) -> GlobalTransform {
    transform * GlobalTransform::from(
        Transform::from_translation(aabb.center.into())
            .with_scale((aabb.half_extents * 2.).into()),
    )
}

pub(crate) fn aabb_contains_point(aabb: &Aabb, point: Vec3) -> bool {
    point.x >= aabb.min().x && point.x <= aabb.max().x &&
    point.y >= aabb.min().y && point.y <= aabb.max().y &&
    point.z >= aabb.min().z && point.z <= aabb.max().z
}

pub(crate) fn triangle_intersects_aabb(v0: Vec3, v1: Vec3, v2: Vec3, aabb: &Aabb) -> bool {
    let aabb_min = aabb.min();
    let aabb_max = aabb.max();

    if aabb_contains_point(aabb, v0) || aabb_contains_point(aabb, v1) || aabb_contains_point(aabb, v2) {
        return true;
    }

    // Triangle edges
    let f0 = v1 - v0;
    let f1 = v2 - v1;
    let f2 = v0 - v2;

    // Test axes a00..a22 (9 tests)
    let axes = [
        Vec3::new(0.0, -f0.z, f0.y),
        Vec3::new(0.0, -f1.z, f1.y),
        Vec3::new(0.0, -f2.z, f2.y),
        Vec3::new(f0.z, 0.0, -f0.x),
        Vec3::new(f1.z, 0.0, -f1.x),
        Vec3::new(f2.z, 0.0, -f2.x),
        Vec3::new(-f0.y, f0.x, 0.0),
        Vec3::new(-f1.y, f1.x, 0.0),
        Vec3::new(-f2.y, f2.x, 0.0),
    ];

    for axis in &axes {
        if !overlap_on_axis(v0, v1, v2, aabb_min.into(), aabb_max.into(), *axis) {
            return false;
        }
    }

    // Test the AABB's face normals (3 tests)
    let axes = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    for axis in &axes {
        if !overlap_on_axis(v0, v1, v2, aabb_min.into(), aabb_max.into(), *axis) {
            return false;
        }
    }

    // Test the triangle's face normal (1 test)
    let normal = f0.cross(f1);
    if !overlap_on_axis(v0, v1, v2, aabb_min.into(), aabb_max.into(), normal) {
        return false;
    }

    true
}

fn overlap_on_axis(v0: Vec3, v1: Vec3, v2: Vec3, aabb_min: Vec3, aabb_max: Vec3, axis: Vec3) -> bool {
    let (min_t, max_t) = project_triangle(v0, v1, v2, axis);
    let (min_b, max_b) = project_aabb(aabb_min, aabb_max, axis);

    max_t >= min_b && max_b >= min_t
}

fn project_triangle(v0: Vec3, v1: Vec3, v2: Vec3, axis: Vec3) -> (f32, f32) {
    let p0 = v0.dot(axis);
    let p1 = v1.dot(axis);
    let p2 = v2.dot(axis);

    (p0.min(p1.min(p2)), p0.max(p1.max(p2)))
}

fn project_aabb(aabb_min: Vec3, aabb_max: Vec3, axis: Vec3) -> (f32, f32) {
    let vertices = [
        aabb_min,
        Vec3::new(aabb_min.x, aabb_min.y, aabb_max.z),
        Vec3::new(aabb_min.x, aabb_max.y, aabb_min.z),
        Vec3::new(aabb_min.x, aabb_max.y, aabb_max.z),
        Vec3::new(aabb_max.x, aabb_min.y, aabb_min.z),
        Vec3::new(aabb_max.x, aabb_min.y, aabb_max.z),
        Vec3::new(aabb_max.x, aabb_max.y, aabb_min.z),
        aabb_max,
    ];

    let mut min = vertices[0].dot(axis);
    let mut max = min;

    for vertex in &vertices[1..] {
        let projection = vertex.dot(axis);
        if projection < min {
            min = projection;
        }
        if projection > max {
            max = projection;
        }
    }

    (min, max)
}