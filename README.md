# bevy_procedural_grass
[![crates.io](https://img.shields.io/crates/v/bevy_procedural_grass.svg)](https://crates.io/crates/bevy_procedural_grass)
[![Doc](https://docs.rs/bevy_procedural_grass/badge.svg)](https://docs.rs/bevy_procedural_grass)

A plugin for `bevy 0.12` that generates grass on top of any mesh.

![bevy_procedural_grass](https://github.com/jadedbay/bevy_procedural_grass/assets/86005828/6b806f78-0910-40c7-9785-2d4e42d6ebb1)

## Usage

Add `bevy_procedural_grass` dependency to `Cargo.toml`:

```toml
[dependencies]
bevy_procedural_grass = "0.2.0"
```

### Generate grass on top of entity:

```rust
use bevy::prelude::*;
use bevy_procedural_grass::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ProceduralGrassPlugin::default(), // add grass plugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let plane = commands.spawn(
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::default())),
            ..default()
        }, 
    ).id();

    // spawn grass
    commands.spawn(GrassBundle {
        mesh: meshes.add(GrassMesh::mesh(7)), // how many segments you want in the mesh (no. of verts = segments * 2 + 1)
        grass: Grass {
            entity: Some(plane.clone()), // set entity that grass will generate on top of.
            ..default()
        },
        lod: GrassLODMesh::new(meshes.add(GrassMesh::mesh(3))), // optional: enables LOD
        ..default()
    });

    // create object that displaces grass
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial::from(Color::WHITE)),
            ..default()
        },
    ));
}
```

## Features
- Grass positions generated based of mesh
- Wind Animation
- Lighting/Shadows for directional lights
- GPU Instancing
- Frustum/Distance Culling
- LOD

## TODO
- Lighting for point and spot lights (Currently only supports directional lights).
- Improve Animation.
- Grass Clumping for less uniform grass generation.
- Grass Interaction, allow grass to move out of the way of other entites.
- Density Map.
- Compute Shaders, use compute shaders to generate grass instance data each frame to optimize memory usage.

## Resources
- [Modern Foliage Rendering - Acerola](https://www.youtube.com/watch?v=jw00MbIJcrk)
- [Procedural Grass in 'Ghost of Tsushima' - GDC](https://www.youtube.com/watch?v=Ibe1JBF5i5Y)
- [Unity-Grass - cainrademan](https://github.com/cainrademan/Unity-Grass/)
- [warbler_grass - EmiOnGit](https://github.com/EmiOnGit/warbler_grass/)
