use bevy::{math::f32, prelude::*};
use bevy_mod_picking::{PickableBundle, PickingPlugin, DebugCursorPickingPlugin};

const BOARD_HEIGHT: f32 = 0.2;
const BOARD_SIZE: f32 = 4.;

const ROD_HEIGHT: f32 = 0.5;
const ROD_RADIUS: f32 = 0.07;
const SPACE: f32 = BOARD_SIZE / 4.;
const OFFSET: f32 = SPACE / 2.;

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Base
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: 0.,
            max_x: BOARD_SIZE,
            min_y: -BOARD_HEIGHT,
            max_y: 0.,
            min_z: 0.,
            max_z: BOARD_SIZE,
        })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Rods
    let rod_mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Capsule {
        radius: ROD_RADIUS,
        rings: 0,
        depth: 0.5,
        latitudes: 30,
        longitudes: 50,
        uv_profile: shape::CapsuleUvProfile::Uniform,
    }));

    let rod_material: Handle<StandardMaterial> = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    for x in 0u8..4 {
        for y in 0u8..4 {
            spawn_rod(&mut commands, rod_mesh.clone(), rod_material.clone(), (x as f32, y as f32));
        }
    }
}

pub struct Rod {
    pub x: f32,
    pub y: f32,
}

fn spawn_rod(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: (f32, f32)
) {
    commands.spawn_bundle(PbrBundle {
        mesh: mesh,
        material: material,
        transform: Transform::from_xyz(position.0*SPACE+OFFSET, ROD_HEIGHT / 2 as f32, position.1*SPACE+OFFSET),
        ..Default::default()
    })
    .insert_bundle(PickableBundle::default())
    .insert(Rod { x: position.0, y: position.1 });
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(PickingPlugin)
            .add_plugin(DebugCursorPickingPlugin)
            .add_startup_system(create_board.system());
    }
}