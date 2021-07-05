use bevy::prelude::*;

mod board;
use board::*;

mod camera;
use camera::*;

mod winner;

mod constants;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(BoardPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
) {
    let dist: f32 = 6.0;

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(constants::BOARD_SIZE/2.0, 8.0, constants::BOARD_SIZE/2.0),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-dist, 3.0, -dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-dist, 3.0, constants::BOARD_SIZE+dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(constants::BOARD_SIZE+dist, 3.0, -dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(constants::BOARD_SIZE+dist, 3.0, constants::BOARD_SIZE+dist),
        ..Default::default()
    });
}