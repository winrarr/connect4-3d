use core::time;
use std::thread;

use bevy::prelude::*;

mod board;
use board::*;

mod camera;
use camera::*;
use rpc::*;

mod constants;
mod rpc;
mod winner;

use local_ip_address::local_ip;

fn main() {
    game();
}

fn server_client() {
    let addr = (local_ip().unwrap(), 3000);

    thread::spawn(move || {
        server::start(addr).unwrap();
    });

    client::run(addr).unwrap();
    thread::sleep(time::Duration::from_millis(1000));
}

fn game() {
    setup();

    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(BoardPlugin)
        .add_startup_system(setup_lights.system())
        .add_state(AppState::Menu)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
}

fn setup() {
    std::env::set_var("BEVY_WGPU_BACKEND", "vulkan")
}

fn setup_lights(mut commands: Commands) {
    let dist: f32 = 6.0;

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-dist, 3.0, -dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-dist, 3.0, constants::BOARD_SIZE + dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(constants::BOARD_SIZE + dist, 3.0, -dist),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(
            constants::BOARD_SIZE + dist,
            3.0,
            constants::BOARD_SIZE + dist,
        ),
        ..Default::default()
    });
}
