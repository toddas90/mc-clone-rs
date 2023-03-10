use bevy::prelude::*;
use rand::random;
use noise::{Perlin, Fbm};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder, NoiseMap};

#[derive(Component)]
pub struct Chunks {
    chunks: Vec<Chunk>,
}

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

#[derive(Component)]
pub struct Chunk {
    blocks: Vec<Block>,
    height_map: NoiseMap,
    contains_player: bool,
}

impl Chunk {
    pub fn new(
        start_pos: i32,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,) -> Self {
        let mut blocks = Vec::new();

        let SIZE = 16 + start_pos;

        let fbm = Fbm::<Perlin>::new(rand::random());

        let height_map = PlaneMapBuilder::<_, 2>::new(&fbm)
                .set_size(1024, 1024)
                .set_x_bounds(-5.0, 5.0)
                .set_y_bounds(-5.0, 5.0)
                .build();

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
            height_map,
            contains_player: false,
        }
    }

    pub fn player_in_chunk(
        &mut self,
        player: &Query<&Transform, With<Camera>>,
    ) {
        for transform in self.blocks.iter_mut() {
            for player_transform in player.iter() {
                let distance = transform.object.transform.translation.distance(player_transform.translation);
                if distance < 16.0 {
                    self.contains_player = true;
                }
            }
        }
        self.contains_player = false;
    }
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

// Remove Chunks that are more than 4 chunks away from the camera/player in each direction
pub fn chunk_cleanup(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut Chunk), With<Chunks>>,
    player: Query<&Transform, With<Camera>>,
) {
    for (_, mut chunk) in chunks.iter_mut() {
        chunk.player_in_chunk(&player);
    }

    for player_transform in player.iter() {
        let x = player_transform.translation.x;
        let z = player_transform.translation.z;

        let mut x_chunk = x / 16.0;
        let mut z_chunk = z / 16.0;

        if x_chunk < 0.0 {
            x_chunk -= 1.0;
        }
        if z_chunk < 0.0 {
            z_chunk -= 1.0;
        }

        let x_chunk = x_chunk as i32;
        let z_chunk = z_chunk as i32;

        for chunk in chunks.iter() {
            let chunk_x = chunk.1.blocks[0].object.transform.translation.x;
            let chunk_z = chunk.1.blocks[0].object.transform.translation.z;

            if chunk_x < x_chunk as f32 - 4.0 || chunk_x > x_chunk as f32 + 4.0 || chunk_z < z_chunk as f32 - 4.0 || chunk_z > z_chunk as f32 + 4.0 {
                commands.entity(chunk.0).despawn_recursive();
            }
        }
    }
}

// Add Chunks that are less than 4 chunks away from the camera/player and that don't already exist
pub fn chunk_generation(
    mut commands: Commands,
    chunks: Query<(Entity, &mut Chunk), With<Chunks>>,
    player: Query<&Transform, With<Camera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
        // From the chunk containing the player, generate chunks in each direction until 4 chunks away
        // Unless they already exist
        for player_transform in player.iter() {
            let x = player_transform.translation.x;
            let z = player_transform.translation.z;

            let mut x_chunk = x / 16.0;
            let mut z_chunk = z / 16.0;

            if x_chunk < 0.0 {
                x_chunk -= 1.0;
            }
            if z_chunk < 0.0 {
                z_chunk -= 1.0;
            }

            let x_chunk = x_chunk as i32;
            let z_chunk = z_chunk as i32;

            for x in x_chunk..x_chunk + 4 {
                for z in z_chunk..z_chunk + 4 {
                    let chunk = Chunk::new(16 * x, &mut meshes, &mut materials);
                    commands.spawn_batch(chunk.blocks);
                }
            }
        }
}

pub fn init_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
        // Generate the first four chunks
        let mut initial = Chunks {
            chunks: Vec::new(),
        };

        for i in 0..4 {
            let chunk = Chunk::new(16 * i, &mut meshes, &mut materials);
            initial.chunks.push(chunk);
        }

        // Spawn the chunks
        for chunk in initial.chunks {
            commands.spawn_batch(chunk.blocks);
        }
}