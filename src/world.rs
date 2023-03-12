use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, Arc};
use bevy::prelude::*;
use bevy::utils::HashSet;
use noise::{Perlin, Fbm};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use rayon::prelude::*;

const CHUNK_SIZE: i32 = 4;
const VIEW_DISTANCE: i32 = 4;
const GENERATION_RADIUS: i32 = 4;
const CACHE_SIZE: i32 = 4;
const SEED : u32 = 69;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}

impl Block {
    fn new(position: IVec3) -> Self {
        Self {
            position: Position::new(position.x, position.y, position.z),
        }
    }

    fn is_visible(self, camera_pos: IVec3) -> bool {
        let x = self.position.x;
        let y = self.position.y;
        let z = self.position.z;
        let cx = camera_pos.x;
        let cy = camera_pos.y;
        let cz = camera_pos.z;
        let x_diff = (x - cx).abs();
        let y_diff = (y - cy).abs();
        let z_diff = (z - cz).abs();
        x_diff < VIEW_DISTANCE && y_diff < VIEW_DISTANCE && z_diff < VIEW_DISTANCE
    }
}

impl Hash for Chunk {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        let mut hasher = DefaultHasher::new();
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

        // let blocks = HashSet::new();
                
        // for x in 0..CHUNK_SIZE {
        //     for z in 0..CHUNK_SIZE {
        //         let noise = height_map.get_value(x as usize, z as usize);
        //         let height = (noise * 10.0) as i32;
        //         for y in 0..height {
        //             let block = Block::new(IVec3::new(x, y, z));
        //             blocks.insert(block);
        //         }
        //     }
        // }
        Self {
            blocks: blocks.clone(),
            center_pos,
        }
    }

    fn is_visible(&self, camera_pos: IVec3) -> bool {
        let x = self.center_pos.x;
        let z = self.center_pos.y;
        let cx = camera_pos.x / CHUNK_SIZE;
        let cz = camera_pos.z / CHUNK_SIZE;
        let x_diff = (x - cx).abs();
        let z_diff = (z - cz).abs();
        x_diff < VIEW_DISTANCE && z_diff < VIEW_DISTANCE
    }
}

impl ChunkManager {  
    pub fn update(&mut self, camera_pos: IVec3) {
        let chunk_x = camera_pos.x / CHUNK_SIZE;
        let chunk_z = camera_pos.z / CHUNK_SIZE;

        // Remove chunks that are no longer visible.
        self.visible_chunks.retain(|chunk| chunk.is_visible(camera_pos));

        // If a chunk in the cache is now visible, move it to the visible chunks.
        for chunk in self.chunk_cache.iter() {
            if chunk.is_visible(camera_pos) {
                self.visible_chunks.insert(chunk.clone());
            }
        }

        self.chunk_cache.retain(|chunk| {
            !self.visible_chunks.contains(chunk)
        });
        
        // ----- Parallel version of the commented code. Not sure if it works properly yet. -----
        let visible_chunks_mutex = Mutex::new(self.visible_chunks.clone());
        (-GENERATION_RADIUS..GENERATION_RADIUS).into_par_iter().for_each(|x| {
            (-GENERATION_RADIUS..GENERATION_RADIUS).into_par_iter().for_each(|z| {
                if visible_chunks_mutex.lock().unwrap().contains(&Chunk::new(IVec2::new(chunk_x + x, chunk_z + z))) {
                    return;
                }
                let chunk = Chunk::new(IVec2::new(chunk_x + x, chunk_z + z));
                if chunk.is_visible(camera_pos) {
                    visible_chunks_mutex.lock().unwrap().insert(chunk);
                }
            });
        });
        self.visible_chunks = visible_chunks_mutex.into_inner().unwrap();
    }

    pub fn load_chunks(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        camera_pos: IVec3,
    ) {
        // ----- Parallel version of the commented code. Not sure if it works properly yet. -----
        // Convert HashSet to Vec
        let visible_chunks_vec: Vec<&Chunk> = self.visible_chunks.iter().collect();

        // Create a mutex for all mutable data
        let shared_data = Arc::new(Mutex::new((commands, meshes, materials)));

        // Parallelize loop over visible chunks
        visible_chunks_vec.par_iter().for_each(|chunk| {
            // Sequential loop over blocks in each chunk
            for block in chunk.blocks.iter() {
                if block.is_visible(camera_pos) {
                    // Lock the mutex to access the mutable data
                    let mut data = shared_data.lock().unwrap();
                    let (commands, meshes, materials) = &mut *data;

                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
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
        });
    }

    pub fn unload_chunks(
        &self,
        commands: &mut Commands,
        camera_pos: IVec3,
        query: Query<Entity, With<Block>>,
    ) {
        for entity in query.iter() {
            let block = query.get(entity).unwrap();
            let object: &Block = query.get_component(entity).unwrap();
            if !object.is_visible(camera_pos) {
                commands.entity(block).despawn();
            }
        }
    }
}

// impl ChunkManager {  
//     pub fn update(&mut self, camera_pos: IVec3) {
//         let chunk_x = camera_pos.x / CHUNK_SIZE;
//         let chunk_z = camera_pos.z / CHUNK_SIZE;
//         let new_visible_chunks = Mutex::new(HashSet::new());
//         let new_chunk_cache = HashSet::new();
    
//         let chunks = (0..GENERATION_RADIUS * 2)
//             .flat_map(|x| (0..GENERATION_RADIUS * 2).map(move |z| (x - GENERATION_RADIUS, z - GENERATION_RADIUS)))
//             .collect::<Vec<_>>();
    
//         chunks.par_iter().for_each(|(x, z)| {
//             for chunk in self.visible_chunks.iter().chain(self.chunk_cache.iter()) {
//                 if chunk.center_pos.x == chunk_x + x && chunk.center_pos.y == chunk_z + z {
//                     return;
//                 }
//             }
//             let chunk = Chunk::new(IVec2::new(chunk_x + x, chunk_z + z));
//             new_visible_chunks.lock().unwrap().insert(chunk);
//         });
    
//         self.visible_chunks.extend(new_visible_chunks.lock().unwrap().iter().cloned());
//         self.chunk_cache.extend(new_chunk_cache);
//     }

//     pub fn load_chunks(
//         &self,
//         commands: &mut Commands,
//         meshes: &mut ResMut<Assets<Mesh>>,
//         materials: &mut ResMut<Assets<StandardMaterial>>,
//         camera_pos: IVec3,
//     ) {
//         // MAKE THIS PARALLEL!!!
//         // Render the visible blocks in the visible chunks
//         for chunk in self.visible_chunks.iter() {
//             for block in chunk.blocks.iter() {
//                 if block.is_visible(camera_pos) {
//                     commands.spawn(PbrBundle {
//                         mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
//                         material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
//                         transform: Transform::from_translation(Vec3::new(
//                             block.position.x as f32,
//                             block.position.y as f32,
//                             block.position.z as f32,
//                         )),
//                         ..Default::default()
//                     });
//                 }
//             }
//         }
//     }

//     pub fn unload_chunks(
//         &self,
//         commands: &mut Commands,
//         camera_pos: IVec3,
//         query: Query<Entity, With<Block>>,
//     ) {
//         for entity in query.iter() {
//             let block = query.get(entity).unwrap();
//             let object: &Block = query.get_component(entity).unwrap();
//             if !object.is_visible(camera_pos) {
//                 commands.entity(block).despawn();
//             }
//         }
//     }
// }

pub fn init_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    chunk_manager.update(IVec3::new(0, 0, 0));
    chunk_manager.load_chunks(&mut commands, &mut meshes, &mut materials, IVec3::new(1, 1, 1));
    println!("World initialized");
}

pub fn update_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
    camera: Query<&Transform, With<Camera3d>>,
    entities: Query<Entity, With<Block>>,
) {
    // Get camera position
    let pos = camera.single();
    let pos = IVec3::new(
        pos.translation.x.floor() as i32,
        pos.translation.y.floor() as i32,
        pos.translation.z.floor() as i32,
    );

    // Update the chunk data
    chunk_manager.update(pos);
    chunk_manager.unload_chunks(&mut commands, pos, entities);
    chunk_manager.load_chunks(&mut commands, &mut meshes, &mut materials, pos);
    println!("World updated");
}