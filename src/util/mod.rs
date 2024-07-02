use bevy::{gizmos::aabb, prelude::*, render::primitives::Aabb};

use crate::grass::Grass;

#[allow(dead_code)]
pub(crate) fn aabb_transform(aabb: Aabb, transform: GlobalTransform) -> GlobalTransform {
    transform * GlobalTransform::from(
        Transform::from_translation(aabb.center.into())
            .with_scale((aabb.half_extents * 2.).into()),
    )
}

#[allow(dead_code)]
pub fn draw_chunks(
    mut gizmos: Gizmos,
    query: Query<&Grass>,
) {
    for grass in query.iter() {
        for chunk in &grass.chunks {
            gizmos.cuboid(aabb_transform(chunk.aabb, GlobalTransform::IDENTITY), Color::WHITE);
        }
    }
}