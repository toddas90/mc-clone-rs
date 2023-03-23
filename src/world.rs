use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

const CHUNK_SIZE: i32 = 16;
const SEED: u32 = 69;
const BLOCK_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

// ---------- Block ----------
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Block {
    position: IVec3,
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self { position }
    }

    fn get_pos(&self) -> IVec3 {
        self.position
    }
}
// --------------------------

// ---------- Chunk ----------
#[derive(Component, Clone)]
pub struct Chunk {
    blocks: HashSet<Block>,
    mesh: Handle<Mesh>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            blocks: HashSet::new(),
            mesh: Default::default(),
        }
    }

    fn gen_blocks(&mut self, noise: &NoiseMap) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = noise.get_value(x as usize, z as usize) * 10.0;
                for y in 0..height as i32 {
                    let block_pos = IVec3::new(x, y, z);
                    let block = Block::new(block_pos);
                    self.blocks.insert(block);
                }
            }
        }
    }

    fn gen_mesh(&mut self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut verticies = Vec::new();
        let mut indicies = Vec::new();

        let visible_blocks = self
            .blocks
            .par_iter()
            .filter(|block| {
                let block_pos = block.position;
                let block_pos =
                    Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
                let block_pos = block_pos + BLOCK_SIZE;

                // Check if the block is surrounded by other blocks
                // If it is, don't render it
                self.blocks.par_iter().any(|other_block| {
                    let other_block_pos = other_block.position;
                    let other_block_pos = Vec3::new(
                        other_block_pos.x as f32,
                        other_block_pos.y as f32,
                        other_block_pos.z as f32,
                    );
                    let other_block_pos = other_block_pos + BLOCK_SIZE;

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
            let block_pos = block_pos + BLOCK_SIZE;

            let mut block_verticies = Vec::new();
            let block_indicies = vec![
                0, 1, 3, 3, 1, 2, // Front
                1, 5, 2, 2, 5, 6, // Right
                5, 4, 6, 6, 4, 7, // Back
                4, 0, 7, 7, 0, 3, // Left
                3, 2, 7, 7, 2, 6, // Top
                4, 5, 0, 0, 5, 1 // Bottom
            ];

            // Front
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0));

            // Back
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0));

            // Left
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0));

            // Right
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0));

            // Top
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y + 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y + 1.0, block_pos.z + 1.0));

            // Bottom
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z - 1.0));
            block_verticies.push(Vec3::new(block_pos.x + 1.0, block_pos.y - 1.0, block_pos.z + 1.0));
            block_verticies.push(Vec3::new(block_pos.x - 1.0, block_pos.y - 1.0, block_pos.z + 1.0));

            verticies.extend(block_verticies);
            indicies.extend(block_indicies);
        });

        println!("verticies: {}", verticies.len());
        println!("indicies: {}", indicies.len());

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies);
        mesh.set_indices(Some(Indices::U32(indicies)));

        mesh
    }
}
// ---------------------------

// ---------- World ----------
#[derive(Resource)]
pub struct Map {
    chunks: HashMap<IVec2, Chunk>,
    // cache: HashMap<IVec2, Chunk>,
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
            // cache: HashMap::new(),
            noise: height_map,
        }
    }
}
// ---------------------------

// ---------- Systems ----------
pub fn initialize_world(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Generate four chunks.
    // for x in 0..4 {
    //     for y in 0..4 {
    //         let chunk_pos = IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE);
    //         println!("Generating chunk at {:?}", chunk_pos);
    //         let mut chunk = Chunk::new();
    //         chunk.gen_blocks(&map.noise);
    //         chunk.mesh = meshes.add(chunk.gen_mesh());
    //         map.chunks.insert(chunk_pos, chunk);
    //     }
    // }

    // Add the chunk to the world.
    // for (chunk_pos, chunk) in map.chunks.iter() {
    //     commands.spawn(PbrBundle {
    //         mesh: chunk.mesh.clone(),
    //         material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
    //         transform: Transform::from_translation(chunk_pos.as_vec2().extend(0.0)),
    //         ..Default::default()
    //     });
    // }

    // Generate a single chunk.
    let chunk_pos = IVec2::new(0, 0);
    let mut chunk = Chunk::new();
    chunk.gen_blocks(&map.noise);
    chunk.mesh = meshes.add(chunk.gen_mesh());
    map.chunks.insert(chunk_pos, chunk);

    commands.spawn(PbrBundle {
        mesh: map.chunks.get(&chunk_pos).unwrap().mesh.clone(),
        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
        transform: Transform::from_translation(chunk_pos.as_vec2().extend(0.0)),
        ..Default::default()
    });

    // Find the total blocks generated.
    let total_blocks = map
        .chunks
        .iter()
        .fold(0, |acc, (_, chunk)| acc + chunk.blocks.len());

    println!("Total blocks: {}", total_blocks);
}
// -----------------------------
