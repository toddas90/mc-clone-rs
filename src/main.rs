use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

mod world;
use world::*;

// This is a simple example of a camera that flies around.
// There's an included example of a system that toggles the "enabled"
// property of the fly camera with "T"

fn init(mut commands: Commands,) {
	commands.spawn(DirectionalLightBundle {
		transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)),
		..Default::default()
	});
	commands
		.spawn(Camera3dBundle::default())
		.insert(FlyCamera::default());

	println!("Started example!");
}

// Press "T" to toggle keyboard+mouse control over the camera
fn toggle_button_system(
	input: Res<Input<KeyCode>>,
	mut query: Query<&mut FlyCamera>,
) {
	for mut options in query.iter_mut() {
		if input.just_pressed(KeyCode::T) {
			println!("Toggled FlyCamera enabled!");
			options.enabled = !options.enabled;
		}
	}
}

fn main() {
	App::new()
		// .insert_resource(Msaa::Sample4)
		.add_plugins(DefaultPlugins)
		.add_startup_system(init)
		.add_startup_system(init_world)
        // .add_system(chunk_generation)
		// .add_system(chunk_cleanup)
		.add_plugin(FlyCameraPlugin)
		.add_system(toggle_button_system)
		.run();
}
