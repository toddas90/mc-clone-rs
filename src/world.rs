use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

const CHUNK_SIZE: i32 = 16;
const SEED: u32 = 69;

// ---------- Block ----------
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Block {
    position: IVec3,
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self { position }
    }
}
// --------------------------

// ---------- Chunk ----------
#[derive(Component, Clone)]
pub struct Chunk {
    blocks: HashSet<Block>,
    mesh: Mesh,
    start_pos: IVec2,
}

impl Chunk {
    fn new(start_pos: IVec2) -> Self {
        Self {
            blocks: HashSet::new(),
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
            start_pos,
        }
    }

    fn gen_blocks(&mut self, noise: &NoiseMap) {
        println!(
            "Generating blocks for chunk at ({}, {})",
            self.start_pos.x, self.start_pos.y
        );
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = noise.get_value(x as usize, z as usize) * 10.0;
                for y in 0..height as i32 {
                    self.blocks.insert(Block::new(IVec3::new(x, y, z)));
                }
            }
        }
    }

    fn gen_mesh(&mut self) {
        let mut verticies = Vec::new();
        let mut indicies = Vec::new();

        let visible_blocks = self
            .blocks
            .par_iter()
            .filter(|block| {
                let block_pos = block.position;
                let block_pos =
                    Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
                let block_pos = block_pos + Vec3::new(0.5, 0.5, 0.5);

                // Check if the block is surrounded by other blocks
                // If it is, don't render it
                self.blocks.par_iter().any(|other_block| {
                    let other_block_pos = other_block.position;
                    let other_block_pos = Vec3::new(
                        other_block_pos.x as f32,
                        other_block_pos.y as f32,
                        other_block_pos.z as f32,
                    );
                    let other_block_pos = other_block_pos + Vec3::new(0.5, 0.5, 0.5);

                    let distance = block_pos.distance(other_block_pos);
                    distance > 1.0
                })
            })
            .collect::<Vec<_>>();

        // For each visible block, get the verticies and indicies that are not back to back with other blocks.
        // This will result in a smaller mesh, and less draw calls.
        visible_blocks.iter().for_each(|block| {
            let block_pos = block.position;
            let block_pos = Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
            let block_pos = block_pos + Vec3::new(0.5, 0.5, 0.5);

            let mut block_verticies = Vec::new();
            let mut block_indicies = Vec::new();

            // Front
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(0, 0, 1)))
            {
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, 0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            // Back
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(0, 0, -1)))
            {
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, -0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            // Left
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(-1, 0, 0)))
            {
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, -0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            // Right
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(1, 0, 0)))
            {
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, -0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            // Top
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(0, 1, 0)))
            {
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, 0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, 0.5, 0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            // Bottom
            if !self
                .blocks
                .contains(&Block::new(block_pos.as_ivec3() + IVec3::new(0, -1, 0)))
            {
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, -0.5));
                block_verticies.push(block_pos + Vec3::new(0.5, -0.5, 0.5));
                block_verticies.push(block_pos + Vec3::new(-0.5, -0.5, 0.5));

                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 1);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32);
                block_indicies.push(verticies.len() as u32 + 2);
                block_indicies.push(verticies.len() as u32 + 3);
            }

            verticies.extend(block_verticies);
            indicies.extend(block_indicies);
        });

        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies);
        self.mesh.set_indices(Some(Indices::U32(indicies)));
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

        let height_map = PlaneMapBuilder::<_, 2>::new(&fbm)
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
pub fn initialize_world(mut commands: Commands, mut map: ResMut<Map>) {
    // Generate four chunks.
    for x in 0..4 {
        for y in 0..4 {
            let chunk_pos = IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE);
            let mut chunk = Chunk::new(chunk_pos);
            chunk.gen_blocks(&map.noise);
            chunk.gen_mesh();
            map.chunks.insert(chunk_pos, chunk.clone());
            commands.spawn(chunk);
        }
    }

    // Find the total blocks generated.
    let mut total_blocks = 0;
    map.chunks.iter().for_each(|(_, chunk)| {
        total_blocks += chunk.blocks.len();
    });

    println!("Total blocks: {}", total_blocks);
}
// -----------------------------
