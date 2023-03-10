use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy::utils::HashSet;
use noise::{Perlin, Fbm};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder, NoiseMap};

const CHUNK_SIZE: i32 = 16;
const VIEW_DISTANCE: i32 = CHUNK_SIZE * GENERATION_RADIUS;
const GENERATION_RADIUS: i32 = 4;
const CACHE_SIZE: i32 = 8;

#[derive(Resource, Clone)]
pub struct NoiseGenerator {
    noise: Fbm<Perlin>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Block {
    position: Position,
    is_visible: bool,
}

#[derive(Component, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
    blocks: BTreeSet<Block>,
    center_pos: IVec2,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    visible_chunks: HashSet<Chunk>,
    chunk_cache: HashSet<Chunk>,
}

impl FromWorld for NoiseGenerator {
    fn from_world(world: &mut World) -> Self {
        let noise = Fbm::<Perlin>::new(rand::random());
        Self { noise }
    }
}

impl Position {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self {
            position: Position::new(position.x, position.y, position.z),
            is_visible: false,
        }
    }

    fn is_visible(&self, camera_pos: IVec3) -> bool {
        let x = self.position.x;
        let z = self.position.z;
        let chunk_x = camera_pos.x / CHUNK_SIZE;
        let chunk_z = camera_pos.z / CHUNK_SIZE;
        let x = x - chunk_x * CHUNK_SIZE;
        let z = z - chunk_z * CHUNK_SIZE;
        x.abs() < VIEW_DISTANCE && z.abs() < VIEW_DISTANCE
    }
}

impl Chunk {
    pub fn new(
        center_pos: IVec2,
        fbm: &Fbm<Perlin>,
    ) -> Self {
        let mut blocks = BTreeSet::new();

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
                    let block = Block::new(IVec3::new(x, y, z));
                    blocks.insert(block);
                }
            }
        }
        Self {
            blocks,
            center_pos,
        }
    }
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            visible_chunks: HashSet::new(),
            chunk_cache: HashSet::new(),
        }
    }

    pub fn update(
        &mut self,
        camera_pos: IVec3,
        fbm: Res<NoiseGenerator>,
    ) {
        let chunk_x = camera_pos.x / CHUNK_SIZE;
        let chunk_z = camera_pos.z / CHUNK_SIZE;
        let mut new_visible_chunks = HashSet::new();
        for x in -GENERATION_RADIUS..GENERATION_RADIUS {
            for z in -GENERATION_RADIUS..GENERATION_RADIUS {
                let chunk = Chunk::new(IVec2::new(chunk_x + x, chunk_z + z), &fbm.noise);
                new_visible_chunks.insert(chunk);
            }
        }
        let mut new_chunk_cache = HashSet::new();
        for x in -CACHE_SIZE..CACHE_SIZE {
            for z in -CACHE_SIZE..CACHE_SIZE {
                let chunk = Chunk::new(IVec2::new(chunk_x + x, chunk_z + z), &fbm.noise);
                new_chunk_cache.insert(chunk);
            }
        }
        self.visible_chunks = new_visible_chunks;
        self.chunk_cache = new_chunk_cache;
    }

    pub fn render_chunks(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        for chunk in self.visible_chunks.iter() {
            for block in chunk.blocks.iter() {
                if block.is_visible {
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                        transform: Transform::from_translation(Vec3::new(
                            block.position.x as f32,
                            block.position.y as f32,
                            block.position.z as f32,
                        )),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

pub fn init_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    fbm: Res<NoiseGenerator>,
) {

    let mut chunk_manager = ChunkManager::new();
    chunk_manager.update(IVec3::new(0, 0, 0), fbm);
    chunk_manager.render_chunks(&mut commands, &mut meshes, &mut materials);
    commands.insert_resource(chunk_manager);
}

pub fn update_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
    fbm: Res<NoiseGenerator>,
    camera: Query<&Transform, With<Camera>>,
) {
    let mut pos = IVec3::new(0, 0, 0);

    for player_transform in camera.iter() {
        pos = IVec3::new(
            player_transform.translation.x.floor() as i32,
            player_transform.translation.y.floor() as i32,
            player_transform.translation.z.floor() as i32,
        );
    }

    chunk_manager.update(pos, fbm);
    chunk_manager.render_chunks(&mut commands, &mut meshes, &mut materials);
}