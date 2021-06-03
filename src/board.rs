use bevy::{math::f32, prelude::*};
use bevy_mod_picking::*;

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
        material: materials.add(Color::rgb_u8(130, 73, 11).into()),
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

    let rod_material: Handle<StandardMaterial> = materials.add(Color::rgb_u8(130, 73, 11).into());

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

fn color_rods(
    selected_rod: Res<SelectedRod>,
    materials: Res<RodMaterials>,
    mut query: Query<(Entity, &Rod, &mut Handle<StandardMaterial>)>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Get entity under the cursor, if there is one
    let top_entity = match picking_camera_query.iter().last() {
        Some(picking_camera) => match picking_camera.intersect_top() {
            Some((entity, _intersection)) => Some(entity),
            None => None,
        },
        None => None,
    };

    for (entity, _, mut material) in query.iter_mut() {
        // Change the material
        *material = if Some(entity) == top_entity {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_rod.entity {
            materials.selected_color.clone()
        } else {
            materials.base_color.clone()
        };
    }
}

#[derive(Default)]
struct SelectedRod {
    entity: Option<Entity>,
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

struct RodMaterials {
    base_color: Handle<StandardMaterial>,
    highlight_color: Handle<StandardMaterial>,
    selected_color: Handle<StandardMaterial>,
}

impl FromWorld for RodMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        RodMaterials {
            base_color: materials.add(Color::rgb_u8(130, 73, 11).into()),
            highlight_color: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
        }
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<SelectedRod>()
            .init_resource::<RodMaterials>()
            .add_plugin(PickingPlugin)
            .add_plugin(DebugCursorPickingPlugin)
            .add_startup_system(create_board.system())
            .add_system(color_rods.system());
    }
}