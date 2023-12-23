use bevy::{prelude::*, render::primitives::Aabb, math::Vec3A};

#[allow(dead_code)]
pub(crate) fn aabb_transform(aabb: Aabb, transform: GlobalTransform) -> GlobalTransform {
    transform
        * GlobalTransform::from(
            Transform::from_translation(aabb.center.into())
                .with_scale((aabb.half_extents * 2.).into()),
        )
}

#[allow(dead_code)]
pub(crate) fn draw_chunk(gizmos: &mut Gizmos, chunk_coord: &(i32, i32, i32), chunk_size: f32) {
    let aabb = Aabb {
        center: Vec3A::from(((chunk_coord.0 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.1 as f32 * chunk_size + chunk_size / 2.), (chunk_coord.2 as f32 * chunk_size + chunk_size / 2.))),
        half_extents: Vec3A::splat(chunk_size as f32 / 2.0),
    };

    gizmos.cuboid(aabb_transform(aabb.clone(), GlobalTransform::IDENTITY), Color::RED);
} 