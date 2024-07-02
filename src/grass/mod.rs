use bevy::{prelude::*, render::primitives::Aabb};

#[derive(Bundle, Default)]
pub struct GrassBundle {
    grass: Grass,
}

#[derive(Component)]
pub struct Grass {
    pub chunk_size: f32,
    pub chunks: Vec<GrassChunk>,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            chunk_size: 3.0,
            chunks: Vec::new(),
        }
    }
}

pub struct GrassChunk {
    pub aabb: Aabb,
}

pub(crate) fn create_chunks(
    meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut Grass, &Handle<Mesh>)>,
) {
    for (mut grass, mesh) in query.iter_mut() {
        let mesh_aabb = meshes.get(mesh).unwrap().compute_aabb().unwrap();
        let mesh_size = mesh_aabb.max() - mesh_aabb.min();
        let chunk_count = (mesh_size / grass.chunk_size).ceil();

        for x in 0..chunk_count.x as usize {
                for z in 0..chunk_count.z as usize {
                    let min = Vec3::from(mesh_aabb.min()) + Vec3::new(grass.chunk_size * x as f32, 0.0, grass.chunk_size * z as f32);
                    let max = min + Vec3::splat(grass.chunk_size);
                    let aabb = Aabb::from_min_max(min, max);

                    grass.chunks.push(GrassChunk {
                        aabb,
                    });
                }
        }

    }
}