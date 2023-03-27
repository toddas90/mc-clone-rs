use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_fly_camera::FlyCamera;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

const CHUNK_SIZE: i32 = 16;
const SEED: u32 = 14;
const BLOCK_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const RENDER_DISTANCE: i32 = 4; // In chunks

// ---------- Block ----------
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Block {
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
}
// --------------------------

// ---------- Chunk ----------
#[derive(Component, Clone)]
pub struct Chunk {
    blocks: HashSet<Block>,
    pos: IVec2,
}

impl Chunk {
    fn new(pos: IVec2) -> Self {
        Self {
            blocks: HashSet::new(),
            pos: pos,
        }
    }

    fn gen_blocks(&mut self, noise: &NoiseMap) {
        let offset = IVec3::new(self.pos.x, 0, self.pos.y);
        // Using the 3d perlin noise, generate a 3d map of blocks
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let height = noise.get_value(
                        x as usize + offset.x as usize,
                        z as usize + offset.z as usize,
                    ) * 10.0;
                    if (y as f64) < height {
                        let block_pos = IVec3::new(x, y, z) + offset;
                        let block = Block::new(block_pos);
                        self.blocks.insert(block);
                    }
                }
            }
        }
    }

    fn gen_meshes(&mut self, meshes: &mut ResMut<Assets<Mesh>>) {
        let visible_blocks = self
            .blocks
            .par_iter()
            .filter(|block| {
                let block_pos = block.position;
                let other_blocks = &self.blocks;

                let surrounding = vec![
                    Block::new(IVec3::new(block_pos.x - 1, block_pos.y, block_pos.z)),
                    Block::new(IVec3::new(block_pos.x, block_pos.y - 1, block_pos.z)),
                    Block::new(IVec3::new(block_pos.x, block_pos.y, block_pos.z - 1)),
                    Block::new(IVec3::new(block_pos.x + 1, block_pos.y, block_pos.z)),
                    Block::new(IVec3::new(block_pos.x, block_pos.y + 1, block_pos.z)),
                    Block::new(IVec3::new(block_pos.x, block_pos.y, block_pos.z + 1)),
                ];

                if other_blocks.contains(&surrounding[0])
                    && other_blocks.contains(&surrounding[1])
                    && other_blocks.contains(&surrounding[2])
                    && other_blocks.contains(&surrounding[3])
                    && other_blocks.contains(&surrounding[4])
                    && other_blocks.contains(&surrounding[5])
                {
                    false
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();

        let new_meshes = Arc::new(Mutex::new(HashMap::new()));

        // For each visible block, get the verticies and indicies that are not back to back with other blocks.
        // This will result in a smaller mesh, and less draw calls.
        visible_blocks.par_iter().for_each(|block| {
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
                4, 5, 0, 0, 5, 1, // Bottom
            ];

            // Need to figure out an effective way to only render the faces that are visible

            // Front
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));

            // Back
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));

            // Left
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));

            // Right
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));

            // Top
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y + 1.0,
                block_pos.z + 1.0,
            ));

            // Bottom
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z - 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x + 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));
            block_verticies.push(Vec3::new(
                block_pos.x - 1.0,
                block_pos.y - 1.0,
                block_pos.z + 1.0,
            ));

            // In this example, normals and UVs don't matter,
            // so we just use the same value for all of them
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![[0., 1., 0.]; block_verticies.len()],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; block_verticies.len()]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, block_verticies);
            mesh.set_indices(Some(Indices::U32(block_indicies)));
            new_meshes.lock().unwrap().insert(block.position, mesh);
        });

        self.blocks
            .retain(|block| !new_meshes.lock().unwrap().contains_key(&block.position));

        for (position, mesh) in new_meshes.lock().unwrap().iter() {
            self.blocks.insert(Block {
                position: *position,
                mesh: meshes.add(mesh.clone()),
            });
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
pub fn initialize_world(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Generate x*y chunks.
    for x in 0..3 {
        for y in 0..3 {
            let chunk_pos = IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE);
            // println!("Generating chunk at {:?}", chunk_pos);
            let mut chunk = Chunk::new(chunk_pos);
            chunk.gen_blocks(&map.noise);
            chunk.gen_meshes(&mut meshes);
            map.chunks.insert(chunk_pos, chunk);
        }
    }

    // Add some chunks to the cache
    for x in 0..3 {
        for y in 0..3 {
            let chunk_pos = IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE);
            let mut chunk = Chunk::new(chunk_pos);
            chunk.gen_blocks(&map.noise);
            chunk.gen_meshes(&mut meshes);
            map.cache.insert(chunk_pos, chunk);
        }
    }

    spawn_chunks(commands, &map, meshes, materials);

    // // Add the chunks to the world.
    // for (_, chunk) in map.chunks.iter() {
    //     for block in chunk.blocks.iter() {
    //         commands.spawn(PbrBundle {
    //             mesh: block.mesh.clone(),
    //             material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
    //             transform: Transform::from_translation(block.position.as_vec3()),
    //             ..Default::default()
    //         });
    //     }
    // }
}

pub fn update_world(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&Transform, With<Camera3d>>,
    entities: Query<(Entity, &Chunk), With<Chunk>>,
) {
    // In here, I will use the camera's position to determine which chunks to load and unload.
    let camera = camera.single();
    let pos = Vec2::new(camera.translation.x, camera.translation.z);

    let mut temp_cache = HashMap::new();

    // If the chunk is outside of the render distance, unload it.
    map.chunks.iter().for_each(|(chunk_pos, chunk)| {
        if pos.x - chunk_pos.x as f32 > (CHUNK_SIZE * RENDER_DISTANCE) as f32
            || pos.y - chunk_pos.y as f32 > (CHUNK_SIZE * RENDER_DISTANCE) as f32
        {
            temp_cache.insert(chunk_pos.clone(), chunk.clone());
        }
    });

    // Unload the chunks.
    map.chunks
        .retain(|chunk_pos, _| temp_cache.contains_key(chunk_pos) == false);

    // Put the chunks into the cache.
    map.cache.extend(temp_cache);

    // Despawn the chunks.
    for (entity, chunk) in entities.iter() {
        if map.cache.contains_key(&chunk.pos) == true {
            println!("Despawning chunk at {:?}", chunk.pos);
            commands.entity(entity).despawn_recursive();
        }
    }

    // Load the chunks.
    if map.chunks.len() < 9 {
        let chunk_pos = IVec2::new(
            (pos.x / CHUNK_SIZE as f32).floor() as i32 * CHUNK_SIZE,
            (pos.y / CHUNK_SIZE as f32).floor() as i32 * CHUNK_SIZE,
        );

        if map.chunks.contains_key(&chunk_pos) == false {
            if map.cache.contains_key(&chunk_pos) {
                println!("Loading chunk at {:?} from cache", chunk_pos);
                let chunk = map.cache.get(&chunk_pos).unwrap().clone();
                map.chunks.insert(chunk_pos, chunk);
            } else {
                println!("Generating chunk at {:?}", chunk_pos);
                let mut chunk = Chunk::new(chunk_pos);
                chunk.gen_blocks(&map.noise);
                chunk.gen_meshes(&mut meshes);
                map.chunks.insert(chunk_pos, chunk);
            }
        }
        spawn_chunks(commands, &map, meshes, materials);
    }

    // println!(
    //     "Loaded Chunks: {} <-> Cached Chunks: {}",
    //     map.chunks.len(),
    //     map.cache.len()
    // );
}

fn spawn_chunks(
    mut commands: Commands,
    map: &Map,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (_, chunk) in map.chunks.iter() {
        commands
            .spawn(Chunk {
                blocks: chunk.blocks.clone(),
                pos: chunk.pos,
            })
            .with_children(|parent| {
                for block in chunk.blocks.iter() {
                    parent.spawn(PbrBundle {
                        mesh: block.mesh.clone(),
                        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                        transform: Transform::from_translation(block.position.as_vec3()),
                        ..Default::default()
                    });
                }
            })
            .insert(VisibilityBundle::default());
    }
}
// -----------------------------
