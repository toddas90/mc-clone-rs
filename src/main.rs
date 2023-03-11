use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

mod world;
use world::*;

// This is a simple example of a camera that flies around.
// There's an included example of a system that toggles the "enabled"
// property of the fly camera with "T"

fn init(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
        ..Default::default()
    });
    commands
        .spawn(Camera3dBundle::default())
        .insert(FlyCamera::default());

    println!("Started Camera!");
}

fn main() {
    App::new()
        // .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minecraft".to_string(),
                resolution: WindowResolution::new(1920.0, 1080.0),
                mode: WindowMode::Windowed,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
		.add_plugin(FlyCameraPlugin)
		.init_resource::<ChunkManager>()
        .add_startup_system(init)
        .add_startup_system(init_world)
		.add_system(update_world)
        .run();
}
