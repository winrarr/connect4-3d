use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

use crate::constants::BOARD_SIZE;

#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::default(),
        OrbitCamera {
            focus: Vec3::splat(BOARD_SIZE / 2.0).with_y(0.4),
            radius: 7.0,
            yaw: -0.65,
            pitch: 0.45,
        },
    ));
}

fn orbit_camera(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut camera)) = query.single_mut() else {
        return;
    };

    if mouse_buttons.pressed(MouseButton::Right) {
        camera.yaw -= mouse_motion.delta.x * 0.01;
        camera.pitch = (camera.pitch - mouse_motion.delta.y * 0.01).clamp(0.15, 1.35);
    }

    camera.radius = (camera.radius - mouse_scroll.delta.y * 0.35).clamp(3.5, 16.0);

    let horizontal_radius = camera.radius * camera.pitch.cos();
    let offset = Vec3::new(
        horizontal_radius * camera.yaw.sin(),
        camera.radius * camera.pitch.sin(),
        horizontal_radius * camera.yaw.cos(),
    );
    transform.translation = camera.focus + offset;
    transform.look_at(camera.focus, Vec3::Y);
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, orbit_camera);
    }
}
