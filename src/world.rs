use std::hash::{Hash, Hasher};
use fxhash::FxHasher;
use std::sync::{Mutex, Arc};
use bevy::{prelude::*, transform};
use bevy::utils::HashSet;
use noise::{Perlin, Fbm};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use rayon::prelude::*;

const CHUNK_SIZE: i32 = 16;
const VIEW_DISTANCE: i32 = 2;
const GENERATION_RADIUS: i32 = 2;
const SEED : u32 = 69;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Block {
    position: Position,
}

#[derive(Component, Clone)]
pub struct Chunk {
    blocks: HashSet<Block>,
    center_pos: IVec2,
    rendered: bool,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    visible_chunks: HashSet<Chunk>,
    chunk_cache: HashSet<Chunk>,
}

impl Position {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn from_ivec3(pos: IVec3) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self {
            position: Position::new(position.x, position.y, position.z),
        }
    }
}

impl Hash for Chunk {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        let mut hasher = FxHasher::default();
        self.center_pos.hash(&mut hasher);
        hasher.finish();
    }
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.center_pos == other.center_pos
    }
}

impl Eq for Chunk {}

impl Chunk {
    pub fn new(
        center_pos: IVec2,
    ) -> Self {
        let fbm = Fbm::<Perlin>::new(SEED);

        let height_map = PlaneMapBuilder::<_, 2>::new(&fbm)
                .set_size(1024, 1024)
                .set_x_bounds(-5.0, 5.0)
                .set_y_bounds(-5.0, 5.0)
                .build();

        let blocks = Arc::new(Mutex::new(HashSet::new()));

        (0..CHUNK_SIZE).into_par_iter().for_each(|x| {
            (0..CHUNK_SIZE).into_par_iter().for_each(|z| {
                let noise = height_map.get_value(x as usize, z as usize);
                let height = (noise * 10.0) as i32;
        
                for y in 0..height {
                    let block = Block::new(IVec3::new(x, y, z));
                    let mut blocks_guard = blocks.lock().unwrap();
                    blocks_guard.insert(block);
                }
            });
        });

        let blocks = blocks.lock().unwrap();

        Self {
            blocks: blocks.clone(),
            center_pos,
            rendered: false,
        }
    }
}

impl ChunkManager {  
    pub fn update(&mut self, camera_transform: &Transform) {
        let camera_pos = camera_transform.translation;
        let far_pos = camera_transform.forward() * CHUNK_SIZE as f32 * VIEW_DISTANCE as f32;
        let end_pos = camera_pos + far_pos;
        if let Some(block) = raycast(camera_pos, end_pos, self) {
            println!("Block found at {:?}", block.position);
            let chunk_pos = IVec2::new(block.position.x / CHUNK_SIZE, block.position.z / CHUNK_SIZE);
            let chunk = Chunk::new(chunk_pos);
            self.add_visible_chunk(&chunk);
        }
    }

    fn add_visible_chunk(&mut self, chunk: &Chunk) {
        if !self.visible_chunks.contains(&chunk) {
            println!("Visible chunk added");
            self.visible_chunks.insert(chunk.clone());
            self.chunk_cache.insert(chunk.clone());
        }
    }

    pub fn get_block(&self, pos: IVec3) -> Option<&Block> {
        for chunk in self.visible_chunks.iter() {
            for block in chunk.blocks.iter() {
                if block.position == Position::from_ivec3(pos) {
                    return Some(block);
                }
            }
        }
        None
    }

    pub fn get_chunk(&self, pos: IVec2) -> Option<&Chunk> {
        for chunk in self.visible_chunks.iter() {
            if chunk.center_pos == pos {
                return Some(chunk);
            }
        }
        None
    }

    // Using the chunk data and the block positions, create the actual blocks on the screen
    pub fn render_chunk(
        &self,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        chunk: &Chunk,
    ) {
        if chunk.rendered {
            return;
        }

        for block in chunk.blocks.iter() {
            let block_pos = Vec3::new(
                block.position.x as f32,
                block.position.y as f32,
                block.position.z as f32,
            );
            let block_transform = Transform::from_translation(block_pos);
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                    transform: block_transform,
                    ..Default::default()
                })
                .insert(block.clone());
        }

        println!("Chunk rendered")
    }
}

pub fn init_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    // Generate the initial chunks
    let mut chunk = Chunk::new(IVec2::new(0, 0));
    chunk_manager.add_visible_chunk(&chunk);
    chunk_manager.render_chunk(&mut commands, &mut materials, &mut meshes, &chunk);
    println!("World initialized");
}

pub fn update_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
    camera: Query<&Transform, With<Camera3d>>,
) {
    // Get camera position
    let transform = camera.single();

    // Update the chunk data
    chunk_manager.update(transform);
    chunk_manager.visible_chunks.iter().for_each(|chunk| {
        chunk_manager.render_chunk(&mut commands, &mut materials, &mut meshes, chunk);
        chunk.rendered = true;
    });
    // println!("World updated");
}

fn raycast(
    start: Vec3,
    end: Vec3,
    chunk_manager: &ChunkManager,
) -> Option<Block> {
    let direction = (end - start).normalize();
    let mut current_pos = start;
    let mut distance = 0.0;
    loop {
        let block_pos = IVec3::new(
            current_pos.x as i32,
            current_pos.y as i32,
            current_pos.z as i32,
        );
        let block = Block::new(block_pos);
        if chunk_manager.get_block(block_pos).is_some() {
            return Some(block);
        }
        current_pos += direction;
        distance += 1.0;
        if distance > CHUNK_SIZE as f32 {
            break;
        }
    }
    None
}

