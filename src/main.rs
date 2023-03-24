use std::{sync::Arc, f32::consts::PI};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowMode, WindowResolution}, pbr::{DirectionalLightShadowMap, CascadeShadowConfigBuilder},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

mod world;
use world::*;

// This is a simple example of a camera that flies around.
// There's an included example of a system that toggles the "enabled"
// property of the fly camera with "T"

fn init(mut commands: Commands) {
    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 5000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 10.0,
            num_cascades: 4,
            maximum_distance: 100.0,
            ..default()
        }
        .into(),
        ..default()
    });
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 2500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(10.0, 12.0, 10.0),
    //     ..default()
    // });
    commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        }).insert(FlyCamera::default());

    println!("Started Camera!");
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
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
        .init_resource::<Map>()
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        .add_startup_system(init)
        .add_startup_system(initialize_world)
        // .add_system(update_world)
        .run();
}
