use bevy::prelude::*;
use bevy::utils::HashSet;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};

const CHUNK_SIZE: i32 = 16;
const VIEW_DISTANCE: i32 = CHUNK_SIZE * 4;
const GENERATION_RADIUS: i32 = 4;
const BUFFER_ZONE: i32 = 8;

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
    center_pos: IVec2,
}

impl Chunk {
    pub fn new(
        center_pos: IVec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let mut blocks = Vec::new();

        let fbm = Fbm::<Perlin>::new(rand::random());

        let height_map = PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size(1024, 1024)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let noise = height_map.get_value(x as usize, z as usize);
                let height = (noise * 10.0) as i32;
                for y in 0..height {
                    if y == height - 1 {
                        let block =
                            Block::new(BlockType::Grass, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    } else if y > height - 5 {
                        let block =
                            Block::new(BlockType::Dirt, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    } else {
                        let block =
                            Block::new(BlockType::Stone, IVec3::new(x, y, z), meshes, materials);
                        blocks.push(block);
                    }
                }
            }
        }
        Self {
            blocks,
            height_map,
            center_pos,
        }
    }

    fn get_visible_blocks(&self, camera_pos: IVec3, view_distance: i32) -> Vec<Block> {
        let mut visible_blocks = Vec::new();

        for block in &self.blocks {
            if (block.object.transform.translation.x - camera_pos.x as f32).abs()
                < view_distance as f32
                && (block.object.transform.translation.y - camera_pos.y as f32).abs()
                    < view_distance as f32
                && (block.object.transform.translation.z - camera_pos.z as f32).abs()
                    < view_distance as f32
            {
                let new_block = block.clone();
                visible_blocks.push(new_block);
            }
        }

        visible_blocks
    }
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    player_position: Vec3,
    generation_radius: i32,
    buffer_zone: i32,
    loaded_chunks: HashSet<IVec2>,
}

impl ChunkManager {
    pub fn new(player_position: Vec3, generation_radius: i32, buffer_zone: i32) -> Self {
        Self {
            player_position,
            generation_radius,
            buffer_zone,
            loaded_chunks: HashSet::new(),
        }
    }

    pub fn update(&mut self, player_position: Vec3) {
        let player_chunk = IVec2::new(
            player_position.x.floor() as i32 / CHUNK_SIZE as i32,
            player_position.z.floor() as i32 / CHUNK_SIZE as i32,
        );

        for x in player_chunk.x - self.generation_radius..player_chunk.x + self.generation_radius {
            for z in
                player_chunk.y - self.generation_radius..player_chunk.y + self.generation_radius
            {
                let chunk_pos = IVec2::new(x, z);
                if !self.loaded_chunks.contains(&chunk_pos) {
                    // Generate new chunk
                    self.loaded_chunks.insert(chunk_pos);
                }
            }
        }

        let mut chunks_to_remove = Vec::new();
        for chunk_pos in self.loaded_chunks.iter() {
            let dist = ((chunk_pos.x - player_chunk.x).pow(2)
                + (chunk_pos.y - player_chunk.y).pow(2)) as f32;
            if dist > (self.generation_radius + self.buffer_zone).pow(2) as f32 {
                // Chunk is outside buffer zone, remove it
                chunks_to_remove.push(*chunk_pos);
            }
        }

        for chunk_pos in chunks_to_remove {
            self.loaded_chunks.remove(&chunk_pos);
            // Delete chunk
        }
    }
}

impl Block {
    pub fn new(
        block_type: BlockType,
        coordinates: IVec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        match block_type {
            BlockType::Grass => Self {
                block_type: BlockType::Grass,
                object: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.1, 0.53, 0.0).into()),
                    transform: Transform::from_translation(coordinates.as_vec3()),
                    ..Default::default()
                },
            },
            BlockType::Dirt => Self {
                block_type: BlockType::Dirt,
                object: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(1.0, 0.87, 0.71).into()),
                    transform: Transform::from_translation(coordinates.as_vec3()),
                    ..Default::default()
                },
            },
            _ => Self {
                block_type: BlockType::Stone,
                object: PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.29, 0.36, 0.36).into()),
                    transform: Transform::from_translation(coordinates.as_vec3()),
                    ..Default::default()
                },
            },
        }
    }
}

pub fn init_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for x in 0..4 {
        for z in 0..4 {
            let chunk = Chunk::new(IVec2::new(x, z), &mut meshes, &mut materials);
            chunk_manager.loaded_chunks.insert(IVec2::new(x, z));
        }
    }
}

pub fn chunk_generation(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    player: Query<&Transform, With<Camera>>,
    query: Query<&Chunk>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut pos = IVec3::new(0, 0, 0);

    for player_transform in player.iter() {
        pos = IVec3::new(
            player_transform.translation.x.floor() as i32,
            player_transform.translation.y.floor() as i32,
            player_transform.translation.z.floor() as i32,
        );
    }

    chunk_manager.update(pos.as_vec3());

    let mut visible_blocks = Vec::new();

    // Generate the chunks in the chunk manager
    for chunk in chunk_manager.loaded_chunks.iter() {
        let chunk = Chunk::new(*chunk, &mut meshes, &mut materials);
        let visible = chunk.get_visible_blocks(pos, VIEW_DISTANCE);
        visible_blocks.extend(visible);
    }

    commands.spawn_batch(visible_blocks);
}
