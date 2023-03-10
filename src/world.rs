use bevy::prelude::*;
use rand::random;
use noise::{Perlin, Fbm};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder, NoiseMap};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BlockType {
    Grass,
    Dirt,
    Stone,
}

#[derive(Bundle, Clone)]
struct Block {
    block_type: BlockType,
    object: PbrBundle,
}

#[derive(Component, Clone)]
struct Chunk {
    blocks: Vec<Block>,
}

impl Chunk {
    pub fn new(
        start_pos: i32,
        height_map: &NoiseMap,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,) -> Self {
        let mut blocks = Vec::new();

        let SIZE = 16 + start_pos;

        for x in -SIZE..SIZE {
            for z in -SIZE..SIZE {
                let noise = height_map.get_value(x as usize, z as usize);
                let height = (noise * 10.0) as i32;
                for y in 0..height {
                    if y == height - 1 {
                        let block = Block::new(BlockType::Grass, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    } else if y > height - 5 {
                        let block = Block::new(BlockType::Dirt, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    } else {
                        let block = Block::new(BlockType::Stone, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    }
                }
            }
        }
        Self {
            blocks,
        }
    }

// pub fn update_chunks(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut query: Query<(Entity, &Identity, &Name)>) {

//     // if a chunk is more than 4 chunks away from the player, delete it
//     for (entity, id, name) in query.iter_mut() {
//         if id.0 > 4 {
//             commands.entity(entity).despawn();
//         }
//     }

//     // if a chunk is less than 4 chunks away from the player, generate it
//     for i in 0..4 {
//         let mut found = false;
//         for (entity, id, name) in query.iter_mut() {
//             if id.0 == i as u32 {
//                 found = true;
//             }
//         }
//         if !found {
//             let chunk = chunk_gen(i * 16, &height_map, &mut meshes, &mut materials);
//             commands.spawn_batch(chunk);
//         }
//     }
// }
}

impl Block {
    pub fn new(
        block_type: BlockType,
        coordinates: IVec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,)
        -> Self {
        match block_type {
            BlockType::Grass => {
                Self {
                    block_type: BlockType::Grass,
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.1, 0.53, 0.0).into()),
                        transform: Transform::from_translation(coordinates.as_vec3()),
                        ..Default::default()
                    },
                }
            }
            BlockType::Dirt => {
                Self {
                    block_type: BlockType::Dirt,
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(1.0, 0.87, 0.71).into()),
                        transform: Transform::from_translation(coordinates.as_vec3()),
                        ..Default::default()
                    },
                }
            }
            _ => {
                Self {
                    block_type: BlockType::Stone,
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.29, 0.36, 0.36).into()),
                        transform: Transform::from_translation(coordinates.as_vec3()),
                        ..Default::default()
                    },
                }
            }
        }
    }
}

pub fn world_gen(	
    mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,) {
        
    let fbm = Fbm::<Perlin>::new(rand::random());

    let height_map = PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size(1000, 1000)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();

    let mut chunks = Vec::new();

    for i in 0..4 {
        chunks.push(Chunk::new(i * 16, &height_map, &mut meshes, &mut materials));
    }

    for chunk in chunks.iter() {
        for block in chunk.blocks.iter() {
            commands.spawn(block.object.clone());
        }
    }
}

// NEED TO SAVE THE HEIGHT MAP TO A FILE SO THAT IT DOESN'T HAVE TO BE GENERATED EVERY TIME THE GAME IS STARTED