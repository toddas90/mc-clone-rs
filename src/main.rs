use std::f32::consts::PI;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
// use bevy_flycam::PlayerPlugin;

mod world;
use world::*;

mod cam;
use cam::*;

const CHUNK_SIZE: i32 = 16;
const RENDER_DISTANCE: i32 = 4; // In chunks

// This is a simple example of a camera that flies around.
// There's an included example of a system that toggles the "enabled"
// property of the fly camera with "T"

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<Assets<TextureAtlas>>,
) {
    let texture: Handle<Image> = asset_server.load("../resources/alpha_atlas.png");

    // Save the texture handle so we can use it later.
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(16., 16.), 16, 16, None, None);
    atlas.add(texture_atlas);

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: CHUNK_SIZE as f32,
            num_cascades: 4,
            maximum_distance: CHUNK_SIZE as f32 * RENDER_DISTANCE as f32,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minecraft".to_string(),
                resolution: WindowResolution::new(1440.0, 1080.0), // 4:3
                mode: WindowMode::Windowed,
                ..Default::default()
            }),
            ..Default::default()
        }))
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PlayerPlugin)
        .init_resource::<Map>()
        .add_startup_system(init)
        .add_system(update_world)
        .run();
}
