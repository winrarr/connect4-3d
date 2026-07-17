use bevy::prelude::*;

mod board;
use board::*;

mod camera;
use camera::*;

mod network;
use network::*;

mod winner;

mod constants;

fn main() {
    let network_mode = NetworkMode::from_args();

    App::new()
        .insert_resource(network_mode)
        .add_plugins(DefaultPlugins)
        .add_plugins((NetworkPlugin, CameraPlugin, BoardPlugin))
        .add_systems(Startup, setup_lights)
        .run();
}

fn setup_lights(mut commands: Commands) {
    let dist: f32 = 6.0;

    for position in [
        Vec3::new(-dist, 3.0, -dist),
        Vec3::new(-dist, 3.0, constants::BOARD_SIZE + dist),
        Vec3::new(constants::BOARD_SIZE + dist, 3.0, -dist),
        Vec3::new(
            constants::BOARD_SIZE + dist,
            3.0,
            constants::BOARD_SIZE + dist,
        ),
    ] {
        commands.spawn((PointLight::default(), Transform::from_translation(position)));
    }
}
