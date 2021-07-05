use bevy::{math::f32, prelude::*};
use bevy_mod_picking::*;

use crate::constants;

#[derive(Clone, Copy)]
pub enum PlayerColor {
    Red,
    Blue,
}

struct Board(pub [[Vec<PlayerColor>; 4]; 4]);
impl Default for Board {
    fn default() -> Self {
        Self(Default::default())
    }
}

struct PlayerTurn(pub PlayerColor);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PlayerColor::Red)
    }
}
impl PlayerTurn {
    fn change(&mut self) {
        self.0 = match self.0 {
            PlayerColor::Red => PlayerColor::Blue,
            PlayerColor::Blue => PlayerColor::Red,
        }
    }
}

struct Winner(pub Option<PlayerColor>);
impl Default for Winner {
    fn default() -> Self {
        Self(None)
    }
}

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    
    // Base
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box {
            min_x: 0.,
            max_x: constants::BOARD_SIZE,
            min_y: -constants::BOARD_HEIGHT,
            max_y: 0.,
            min_z: 0.,
            max_z: constants::BOARD_SIZE,
        })),
        material: materials.add(Color::rgb_u8(130, 73, 11).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Rods
    let rod_mesh: Handle<Mesh> = meshes.add(Mesh::from(shape::Capsule {
        radius: constants::ROD_RADIUS,
        rings: 0,
        depth: constants::ROD_HEIGHT,
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

fn select_rod(
    mut commands: Commands,
    piece: Res<PieceMaterialsAndMeshes>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut rods_query: Query<&Rod>,
    mut turn: ResMut<PlayerTurn>,
    mut board: ResMut<Board>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    // Add a piece to clicked rod
    if let Some(picking_camera) = picking_camera_query.iter().last() {
        if let Some((rod_entity, _intersection)) = picking_camera.intersect_top() {
            if let Ok(rod) = rods_query.get_mut(rod_entity) {
                let pieces = &mut board.0[rod.x as usize][rod.y as usize];
                spawn_piece(
                    &mut commands,
                    piece.mesh.clone(), 
                    match turn.0 {
                        PlayerColor::Red => piece.red_material.clone(),
                        PlayerColor::Blue => piece.blue_material.clone(),
                    }, 
                    (rod.x, pieces.len() as f32, rod.y)
                );
                pieces.push(turn.0);
                crate::winner::check_winner(&board.0);
                turn.change();
            }
        }
    }
}

fn spawn_piece(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: (f32, f32, f32)
) {
    commands.spawn_bundle(PbrBundle {
        mesh,
        material,
        transform: Transform::from_xyz(
            position.0*constants::SPACE + constants::OFFSET,
            (2.0*position.1+1.0)*constants::PIECE_RING_RADIUS,
            position.2*constants::SPACE + constants::OFFSET,
        ),
        ..Default::default()
    });
}

pub struct Rod {
    pub x: f32,
    pub y: f32,
}

fn color_rods(
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
        }  else {
            materials.base_color.clone()
        };
    }
}

fn spawn_rod(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: (f32, f32)
) {
    commands.spawn_bundle(PbrBundle {
        mesh,
        material,
        transform: Transform::from_xyz(
            position.0*constants::SPACE+constants::OFFSET,
            constants::ROD_HEIGHT / 2.0,
            position.1*constants::SPACE+constants::OFFSET,
        ),
        ..Default::default()
    })
    .insert_bundle(PickableBundle::default())
    .insert(Rod { x: position.0, y: position.1 });
}

struct RodMaterials {
    base_color: Handle<StandardMaterial>,
    highlight_color: Handle<StandardMaterial>,
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
        }
    }
}

struct PieceMaterialsAndMeshes {
    mesh: Handle<Mesh>,
    red_material: Handle<StandardMaterial>,
    blue_material: Handle<StandardMaterial>,
}

impl FromWorld for PieceMaterialsAndMeshes {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut meshes = world
            .get_resource_mut::<Assets<Mesh>>()
            .unwrap();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        PieceMaterialsAndMeshes {
            mesh: meshes.add(Mesh::from(shape::Torus {
                radius: constants::PIECE_RADIUS,
                ring_radius: constants::PIECE_RING_RADIUS,
                subdivisions_segments: constants::PIECE_SUBDIVISIONS_SEGMENTS,
                subdivisions_sides: constants::PIECE_SUBDIVISIONS_SIDES,
            })),
            red_material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            blue_material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
        }
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<RodMaterials>()
            .init_resource::<PieceMaterialsAndMeshes>()
            .init_resource::<PlayerTurn>()
            .init_resource::<Board>()
            .init_resource::<Winner>()
            .add_plugin(PickingPlugin)
            .add_startup_system(create_board.system())
            .add_system(select_rod.system())
            .add_system(color_rods.system());
    }
}