use bevy::prelude::*;
use bevy_orbit_controls::*;

fn setup_camera(
    mut commands: Commands,
) {
    commands
    .spawn()
    .insert_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(
            Vec3::new(1.5, 0., 1.5),
            Vec3::Y
        ),
        ..Default::default()
    })
    .insert(OrbitCamera::new(5., Vec3::new(1.5, 0., 1.5)));
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(OrbitCameraPlugin)
            .add_startup_system(setup_camera.system());
    }
}