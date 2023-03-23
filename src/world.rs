use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

const CHUNK_SIZE: i32 = 16;
const SEED: u32 = 69;
const BLOCK_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

// ---------- Block ----------
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
struct Block {
    position: IVec3,
    mesh: Handle<Mesh>,
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self {
            position,
            mesh: Default::default(),
        }
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
}

impl Chunk {
    fn new() -> Self {
        Self {
            blocks: HashSet::new(),
        }
    }

    fn gen_blocks(&mut self, noise: &NoiseMap) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = noise.get_value(x as usize, z as usize) * 10.0;
                for y in -2..height as i32 {
                    let block_pos = IVec3::new(x, y, z);
                    let block = Block::new(block_pos);
                    self.blocks.insert(block);
                }
            }
        }
    }

    fn gen_meshes(&mut self, meshes: &mut ResMut<Assets<Mesh>>) {
        let visible_blocks = self
            .blocks
            .par_iter()
            .filter(| block| {
                let block_pos = block.position;

                // Check if the block is surrounded by other blocks
                // If it is, don't render it
                self.blocks.par_iter().any(|other_block| {
                    let other_block_pos = other_block.position;

                    // Use the manhattan distance to check if the block is surrounded
                    let distance = (block_pos.x - other_block_pos.x).abs() + (block_pos.y - other_block_pos.y).abs() + (block_pos.z - other_block_pos.z).abs();
                    distance > 1
                })
            })
            .collect::<Vec<_>>();

        let mut new_meshes = HashMap::new();

        // For each visible block, get the verticies and indicies that are not back to back with other blocks.
        // This will result in a smaller mesh, and less draw calls.
        visible_blocks.iter().for_each(|block| {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

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

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, block_verticies);
            mesh.set_indices(Some(Indices::U32(block_indicies)));
            // In this example, normals and UVs don't matter,
            // so we just use the same value for all of them
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 24]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 24]);

            new_meshes.insert(block.position, mesh);
        });

        // Remove the blocks from self.blocks that are in new_meshes
        self.blocks.retain(|block| !new_meshes.contains_key(&block.position));

        for (position, mesh) in new_meshes {
            self.blocks.insert(Block {
                position,
                mesh: meshes.add(mesh),
            });
        }
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
    for x in 0..4 {
        for y in 0..4 {
            let chunk_pos = IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE);
            println!("Generating chunk at {:?}", chunk_pos);
            let mut chunk = Chunk::new();
            chunk.gen_blocks(&map.noise);
            chunk.gen_meshes(&mut meshes);
            map.chunks.insert(chunk_pos, chunk);
        }
    }

    // Add the chunk to the world.
    for (_chunk_pos, chunk) in map.chunks.iter() {
        for block in chunk.blocks.iter() {
            commands
                .spawn(PbrBundle {
                    mesh: block.mesh.clone(),
                    material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                    transform: Transform::from_translation(block.position.as_vec3()),
                    ..Default::default()
                });
        }
    }

    // Find the total blocks generated.
    let total_blocks = map
        .chunks
        .iter()
        .fold(0, |acc, (_, chunk)| acc + chunk.blocks.len());

    println!("Total blocks: {}", total_blocks);
}
// -----------------------------
