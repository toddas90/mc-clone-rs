use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
// use bevy_flycam::FlyCam;
use cam::*;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::cam;

const CHUNK_SIZE: i32 = 32;
const SEED: u32 = 14;
const BLOCK_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const RENDER_DISTANCE: i32 = 3; // In chunks
const WATER_LEVEL: i32 = 7;

// ---------- Block ----------
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Block {
    mesh: Handle<Mesh>,
    btype: BlockType,
}

impl Block {
    fn new(btype: BlockType) -> Self {
        Self {
            mesh: Default::default(),
            btype,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BlockType {
    Grass,
    Dirt,
    Stone,
    Water,
    Bedrock,
    Air,
}

impl BlockType {
    fn get_color(&self) -> Color {
        match self {
            // Colors from https://encycolorpedia.com/
            BlockType::Grass => Color::hex("91cb7d").unwrap(),
            BlockType::Dirt => Color::hex("9b7653").unwrap(),
            BlockType::Stone => Color::hex("9f9484").unwrap(),
            BlockType::Water => Color::hex("497786").unwrap(),
            BlockType::Bedrock => Color::hex("4d4e52").unwrap(),
            BlockType::Air => Color::hex("000000").unwrap(),
        }
    }
}
// --------------------------

// ---------- Chunk ----------
#[derive(Component, Clone)]
pub struct Chunk {
    blocks: HashMap<IVec3, Block>,
    pos: IVec2,
}

impl Chunk {
    fn new(pos: IVec2) -> Self {
        Self {
            blocks: HashMap::new(),
            pos,
        }
    }

    fn gen_blocks(&mut self, noise: &NoiseMap) {
        let offset = IVec3::new(self.pos.x, 0, self.pos.y);

        let blocks_mutex = Arc::new(Mutex::new(HashMap::new()));

        // Iterate through the blocks and create them. Meshes are handled elsewhere.
        // (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
        //     .into_par_iter()
        //     .rev()
        //     .for_each(|i| {
        //         let x = i % CHUNK_SIZE;
        //         let z = (i / CHUNK_SIZE) % CHUNK_SIZE;
        //         let y = i / (CHUNK_SIZE * CHUNK_SIZE);
        //         let height = noise.get_value((x + offset.x) as usize, (z + offset.z) as usize)
        //             * CHUNK_SIZE as f64;
        //         if (y as f64) < height {
        //             let block_pos = IVec3::new(x, y % CHUNK_SIZE, z) + offset;
        //             let upper_block_pos = IVec3::new(x, (y + 1) % CHUNK_SIZE, z) + offset;
        //             let block = {
        //                 if y == 0 {
        //                     Block::new(BlockType::Bedrock)
        //                 } else if y < 3 && self.blocks.contains_key(&upper_block_pos) {
        //                     Block::new(BlockType::Stone)
        //                 } else if y < 5 && self.blocks.contains_key(&upper_block_pos) {
        //                     Block::new(BlockType::Dirt)
        //                 } else {
        //                     Block::new(BlockType::Grass)
        //                 }
        //             };
        //             let mut blocks = blocks_mutex.lock().unwrap();
        //             blocks.insert(block_pos, block);
        //         }
        //     });

        (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .into_par_iter()
            .for_each(|i| {
                let x = i % CHUNK_SIZE;
                let z = (i / CHUNK_SIZE) % CHUNK_SIZE;
                let y = i / (CHUNK_SIZE * CHUNK_SIZE);
                let height = noise.get_value((x + offset.x) as usize, (z + offset.z) as usize)
                    * CHUNK_SIZE as f64;

                let block_pos = IVec3::new(x, y, z) + offset;

                let mut blocks = blocks_mutex.lock().unwrap();

                // Check if a block already exists at the current position
                if blocks.get(&block_pos).is_some() {
                    return;
                }

                if (y as f64) < height.abs() {
                    let block = if y < 4 {
                        Block::new(BlockType::Stone)
                    } else if y < 7 {
                        Block::new(BlockType::Dirt)
                    } else {
                        Block::new(BlockType::Grass)
                    };
                    blocks.insert(block_pos, block);
                } else if y == WATER_LEVEL {
                    let block = Block::new(BlockType::Water);
                    blocks.insert(block_pos, block);
                }
            });

        self.blocks
            .extend(Arc::try_unwrap(blocks_mutex).unwrap().into_inner().unwrap());
    }

    fn gen_meshes(&mut self, meshes: &mut ResMut<Assets<Mesh>>) {
        // Find the blocks that are not buried.
        let temp = self.blocks.clone();
        let visible_blocks = temp
            .par_iter()
            .filter(|block| {
                let block_pos = block.0;
                let other_blocks = &self.blocks;

                let surrounding = vec![
                    IVec3::new(block_pos.x - 1, block_pos.y, block_pos.z),
                    IVec3::new(block_pos.x, block_pos.y - 1, block_pos.z),
                    IVec3::new(block_pos.x, block_pos.y, block_pos.z - 1),
                    IVec3::new(block_pos.x + 1, block_pos.y, block_pos.z),
                    IVec3::new(block_pos.x, block_pos.y + 1, block_pos.z),
                    IVec3::new(block_pos.x, block_pos.y, block_pos.z + 1),
                ];

                !(other_blocks.contains_key(&surrounding[0])
                    && other_blocks.contains_key(&surrounding[1])
                    && other_blocks.contains_key(&surrounding[2])
                    && other_blocks.contains_key(&surrounding[3])
                    && other_blocks.contains_key(&surrounding[4])
                    && other_blocks.contains_key(&surrounding[5]))
            })
            .collect::<Vec<_>>();

        // Filter out Air blocks.
        let visible_blocks = visible_blocks
            .par_iter()
            .filter(|block| block.1.btype != BlockType::Air)
            .collect::<Vec<_>>();

        let new_meshes = Arc::new(Mutex::new(HashMap::new()));

        // For each visible block, get the verticies and indicies that are not back to back with other blocks.
        visible_blocks.par_iter().for_each(|block| {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

            let block_pos = block.0;
            let block_pos = Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
            let block_pos = block_pos + BLOCK_SIZE;

            let block_indicies = vec![
                0, 1, 3, 3, 1, 2, // Front
                1, 5, 2, 2, 5, 6, // Right
                5, 4, 6, 6, 4, 7, // Back
                4, 0, 7, 7, 0, 3, // Left
                3, 2, 7, 7, 2, 6, // Top
                4, 5, 0, 0, 5, 1, // Bottom
            ];

            // Need to figure out an effective way to only render the faces that are visible
            let block_verticies = vec![
                // Front
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                // Back
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                // Left
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                // Right
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                // Top
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
                // Bottom
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0),
                Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
                Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0),
            ];

            // If water or bedrock, only render the top face
            // if block.1.btype == BlockType::Water || block.1.btype == BlockType::Bedrock {
            //     mesh.insert_attribute(
            //         Mesh::ATTRIBUTE_NORMAL,
            //         vec![
            //             Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
            //             Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0),
            //             Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
            //             Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0),
            //         ],
            //     );

            //     mesh.insert_attribute(
            //         Mesh::ATTRIBUTE_UV_0,
            //         vec![
            //             Vec2::new(0.0, 0.0),
            //             Vec2::new(1.0, 0.0),
            //             Vec2::new(1.0, 1.0),
            //             Vec2::new(0.0, 1.0),
            //         ],
            //     );

            //     mesh.set_indices(Some(Indices::U32(vec![3, 2, 7, 7, 2, 6])));
            //     new_meshes.lock().unwrap().insert(block.0, mesh);
            //     return;
            // }

            // In this example, normals and UVs don't matter,
            // so we just use the same value for all of them
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![[0., 1., 0.]; block_verticies.len()],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1., 1.]; block_verticies.len()]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, block_verticies);
            mesh.set_indices(Some(Indices::U32(block_indicies)));
            new_meshes.lock().unwrap().insert(block.0, mesh);
        });

        // self.blocks
        //     .retain(|pos, block| !new_meshes.lock().unwrap().contains_key(&pos));

        for (position, mesh) in new_meshes.lock().unwrap().iter() {
            // Update the mesh in self.blocks
            let mut block = self.blocks.get(&position).unwrap().to_owned();
            block.mesh = meshes.add(mesh.clone());
            let pos = IVec3::new(position.x, position.y, position.z);
            self.blocks.insert(pos, block.clone());
        }
    }
}
// ---------------------------

// ---------- World ----------
#[derive(Resource)]
pub struct Map {
    chunks: HashMap<IVec2, Chunk>,
    cache: HashMap<IVec2, Chunk>,
    noise: NoiseMap,
}

impl FromWorld for Map {
    fn from_world(_world: &mut World) -> Self {
        let fbm = Fbm::<Perlin>::new(SEED);

        let height_map = PlaneMapBuilder::<_, 3>::new(&fbm)
            .set_size(1024, 1024)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();

        Map {
            chunks: HashMap::new(),
            cache: HashMap::new(),
            noise: height_map,
        }
    }
}
// ---------------------------

// ---------- Systems ----------

// Need ray casting for block addition / deletion. Will do later.

pub fn update_world(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&Transform, With<FlyCam>>,
    entities: Query<(Entity, &Chunk), With<Chunk>>,
) {
    // In here, I will use the camera's position to determine which chunks to load and unload.
    let camera = camera.single();
    let pos = Vec2::new(camera.translation.x, camera.translation.z);

    let mut cached_chunks = Vec::new();

    // Remove chunks outside the render distance and add them to the cache.
    for (chunk_pos, _chunk) in map.chunks.iter() {
        let distance = (chunk_pos.as_vec2() - pos).length();
        if distance > (CHUNK_SIZE * RENDER_DISTANCE) as f32 {
            cached_chunks.push(*chunk_pos);
        }
    }

    // Add the cached chunks to the cache.
    for chunk_pos in cached_chunks.iter() {
        if !map.cache.contains_key(chunk_pos) {
            let chunk = map.chunks.get(chunk_pos).unwrap().clone();
            map.cache.insert(*chunk_pos, chunk);
            map.chunks.remove(chunk_pos);
        }
    }

    // Remove cached chunks that are too far away.
    map.cache.retain(|chunk_pos, _chunk| {
        let distance = (chunk_pos.as_vec2() - pos).length();
        if distance > (CHUNK_SIZE * RENDER_DISTANCE) as f32 {
            cached_chunks.push(*chunk_pos);
            false
        } else {
            true
        }
    });

    // Despawn the chunks.
    for (entity, chunk) in entities.iter() {
        if !map.chunks.contains_key(&chunk.pos) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Load the chunks.
    let player_pos = IVec2::new(
        (pos.x / CHUNK_SIZE as f32).floor() as i32 * CHUNK_SIZE,
        (pos.y / CHUNK_SIZE as f32).floor() as i32 * CHUNK_SIZE,
    );

    // Get chunks around player_pos and put them all in new_chunks.
    let mut new_chunks = vec![
        player_pos,
        player_pos + IVec2::new(CHUNK_SIZE, 0),
        player_pos + IVec2::new(0, CHUNK_SIZE),
        player_pos + IVec2::new(CHUNK_SIZE, CHUNK_SIZE),
        player_pos + IVec2::new(-CHUNK_SIZE, 0),
        player_pos + IVec2::new(0, -CHUNK_SIZE),
        player_pos + IVec2::new(-CHUNK_SIZE, -CHUNK_SIZE),
        player_pos + IVec2::new(-CHUNK_SIZE, CHUNK_SIZE),
        player_pos + IVec2::new(CHUNK_SIZE, -CHUNK_SIZE),
    ];

    // Remove chunks that are already loaded or cached.
    new_chunks.retain(|chunk_pos| !map.chunks.contains_key(chunk_pos));

    // Load the chunks.
    for chunk_pos in new_chunks.iter() {
        // Realized that the perlin noise map required usize coordinates...
        if chunk_pos.x < 0 || chunk_pos.y < 0 {
            continue;
        }

        if !map.chunks.contains_key(chunk_pos) {
            if map.cache.contains_key(chunk_pos) {
                let chunk = map.cache.get(chunk_pos).unwrap().clone();
                map.chunks.insert(*chunk_pos, chunk);
                map.cache.remove(chunk_pos);
            } else {
                let mut chunk = Chunk::new(*chunk_pos);
                chunk.gen_blocks(&map.noise);
                chunk.gen_meshes(&mut meshes);
                map.chunks.insert(*chunk_pos, chunk);
            }
        }
        let chunk = map.chunks.get(chunk_pos).unwrap();
        commands
            .spawn(Chunk {
                blocks: chunk.blocks.clone(),
                pos: chunk.pos,
            })
            .with_children(|parent| {
                for block in chunk.blocks.iter() {
                    parent.spawn(PbrBundle {
                        mesh: block.1.mesh.clone(),
                        material: materials.add(block.1.btype.get_color().into()),
                        ..Default::default()
                    });
                }
            })
            .insert(VisibilityBundle::default());
    }
}
// -----------------------------
