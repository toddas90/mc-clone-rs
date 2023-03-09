use bevy::prelude::*;
use rand::random;
use noise::{NoiseFn, Perlin};

#[derive(Component)]
struct Identity(u32);

#[derive(Component)]
struct Name(String);

#[derive(Bundle)]
struct Block {
    id: Identity,
    name: Name,
    object: PbrBundle,
}

impl Block {
    pub fn new(
        block_type: &str,
        coordinates: (i32, i32, i32),
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,)
        -> Self {
        match block_type {
            "Grass" => {
                Self {
                    id: Identity(random::<u32>()),
                    name: Name("Grass".to_string()),
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.1, 0.53, 0.0).into()),
                        transform: Transform::from_translation(Vec3::new(
                            coordinates.0 as f32, coordinates.1 as f32, coordinates.2 as f32,
                        )),
                        ..Default::default()
                    },
                }
            }
            "Dirt" => {
                Self {
                    id: Identity(random::<u32>()),
                    name: Name("Dirt".to_string()),
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(1.0, 0.87, 0.71).into()),
                        transform: Transform::from_translation(Vec3::new(
                            coordinates.0 as f32, coordinates.1 as f32, coordinates.2 as f32,
                        )),
                        ..Default::default()
                    },
                }
            }
            _ => {
                Self {
                    id: Identity(random::<u32>()),
                    name: Name("Other".to_string()),
                    object: PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                        transform: Transform::from_translation(Vec3::new(
                            coordinates.0 as f32, coordinates.1 as f32, coordinates.2 as f32,
                        )),
                        ..Default::default()
                    },
                }
            }
        }
    }
}

pub fn world_gen(	
    mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,) {
        // let box_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
        // let box_material = materials.add(Color::rgb(1.0, 0.2, 0.3).into());
        
        let perlin = Perlin::new(random::<u32>());
        let mut blocks = Vec::new();

        let SIZE = 50;
        
        for x in -SIZE..SIZE {
            for y in -SIZE/10..SIZE/10 {
                for z in -SIZE..SIZE {
                    let noise = perlin.get([rand::random::<f64>() + x as f64, rand::random::<f64>() + y as f64, rand::random::<f64>() + z as f64]);
                    if noise > 0.65 {
                        let block = Block::new("Grass", (x, y, z), &mut meshes, &mut materials);
                        blocks.push(block);
                    }
                }
            }
        }
        
        commands.spawn_batch(blocks);
    }
