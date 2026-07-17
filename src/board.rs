use bevy::prelude::*;

use crate::{
    constants, network::NetworkCommand, network::NetworkEvent, network::NetworkHandle,
    network::NetworkInbox, network::NetworkReceiveSet, network::NetworkRole,
    network::NetworkStatus, winner,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerColor {
    Red,
    Blue,
}

#[derive(Resource, Default)]
struct Board([[Vec<PlayerColor>; 4]; 4]);

#[derive(Resource)]
struct PlayerTurn(PlayerColor);

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
        };
    }
}

#[derive(Resource, Default)]
struct Winner(Option<PlayerColor>);

#[derive(Component)]
pub struct Rod {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct BoardMaterial(Handle<StandardMaterial>);

impl FromWorld for BoardMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(Color::srgb_u8(135, 48, 0)))
    }
}

#[derive(Resource)]
struct RodMaterials {
    base: Handle<StandardMaterial>,
    highlight: Handle<StandardMaterial>,
}

impl FromWorld for RodMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            base: materials.add(Color::srgb_u8(189, 76, 0)),
            highlight: materials.add(Color::srgb_u8(184, 92, 31)),
        }
    }
}

#[derive(Resource)]
struct PieceMaterialsAndMeshes {
    mesh: Handle<Mesh>,
    red_material: Handle<StandardMaterial>,
    blue_material: Handle<StandardMaterial>,
}

impl FromWorld for PieceMaterialsAndMeshes {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Mesh::from(Torus::new(
                constants::PIECE_RADIUS - constants::PIECE_RING_RADIUS,
                constants::PIECE_RADIUS + constants::PIECE_RING_RADIUS,
            )))
        };

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            mesh,
            red_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                unlit: true,
                ..default()
            }),
            blue_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0),
                unlit: true,
                ..default()
            }),
        }
    }
}

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    board_material: Res<BoardMaterial>,
    rod_materials: Res<RodMaterials>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            constants::BOARD_SIZE,
            constants::BOARD_HEIGHT,
            constants::BOARD_SIZE,
        ))),
        MeshMaterial3d(board_material.0.clone()),
        Transform::from_xyz(
            constants::BOARD_SIZE / 2.0,
            -constants::BOARD_HEIGHT / 2.0,
            constants::BOARD_SIZE / 2.0,
        ),
        Pickable::IGNORE,
    ));

    let rod_mesh = meshes.add(Mesh::from(Capsule3d::new(
        constants::ROD_RADIUS,
        constants::ROD_HEIGHT,
    )));

    for x in 0..4 {
        for y in 0..4 {
            spawn_rod(
                &mut commands,
                rod_mesh.clone(),
                rod_materials.base.clone(),
                rod_materials.highlight.clone(),
                (x, y),
            );
        }
    }
}

fn spawn_rod(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    base_material: Handle<StandardMaterial>,
    highlight_material: Handle<StandardMaterial>,
    position: (usize, usize),
) {
    commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(base_material.clone()),
            Transform::from_xyz(
                position.0 as f32 * constants::SPACE + constants::OFFSET,
                constants::ROD_HEIGHT / 2.0,
                position.1 as f32 * constants::SPACE + constants::OFFSET,
            ),
            Rod {
                x: position.0,
                y: position.1,
            },
        ))
        .observe(update_material_on::<Pointer<Over>>(highlight_material))
        .observe(update_material_on::<Pointer<Out>>(base_material))
        .observe(select_rod);
}

fn update_material_on<E: EntityEvent>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn select_rod(
    press: On<Pointer<Press>>,
    mut commands: Commands,
    piece: Res<PieceMaterialsAndMeshes>,
    network: Res<NetworkHandle>,
    network_status: Res<NetworkStatus>,
    mut turn: ResMut<PlayerTurn>,
    mut board: ResMut<Board>,
    mut game_winner: ResMut<Winner>,
    rods: Query<&Rod>,
) {
    if game_winner.0.is_some() {
        return;
    }

    if let Some(role) = network_status.role {
        let local_color = match role {
            NetworkRole::Host => PlayerColor::Red,
            NetworkRole::Guest => PlayerColor::Blue,
        };
        if !network_status.connected || turn.0 != local_color {
            return;
        }
    }

    let Ok(rod) = rods.get(press.event_target()) else {
        return;
    };

    if place_piece(
        &mut commands,
        &piece,
        &mut board,
        &mut turn,
        &mut game_winner,
        (rod.x, rod.y),
    ) && network_status.role.is_some()
    {
        let _ = network.commands.send(NetworkCommand::Move {
            x: rod.x as u8,
            y: rod.y as u8,
        });
    }
}

fn apply_network_moves(
    mut inbox: ResMut<NetworkInbox>,
    network_status: Res<NetworkStatus>,
    piece: Res<PieceMaterialsAndMeshes>,
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut turn: ResMut<PlayerTurn>,
    mut game_winner: ResMut<Winner>,
) {
    let events = std::mem::take(&mut inbox.events);
    let Some(role) = network_status.role else {
        return;
    };
    if !network_status.connected {
        return;
    }

    let local_color = match role {
        NetworkRole::Host => PlayerColor::Red,
        NetworkRole::Guest => PlayerColor::Blue,
    };

    for event in events {
        let NetworkEvent::Move { x, y } = event else {
            continue;
        };
        if turn.0 == local_color {
            continue;
        }
        if x >= 4 || y >= 4 {
            continue;
        }
        place_piece(
            &mut commands,
            &piece,
            &mut board,
            &mut turn,
            &mut game_winner,
            (x as usize, y as usize),
        );
    }
}

fn place_piece(
    commands: &mut Commands,
    piece: &PieceMaterialsAndMeshes,
    board: &mut Board,
    turn: &mut PlayerTurn,
    game_winner: &mut Winner,
    position: (usize, usize),
) -> bool {
    if game_winner.0.is_some() {
        return false;
    }

    let (x, y) = position;
    let color = turn.0;
    let height = {
        let pieces = &mut board.0[x][y];
        if pieces.len() == 4 {
            return false;
        }
        let height = pieces.len();
        pieces.push(color);
        height
    };

    spawn_piece(
        commands,
        piece.mesh.clone(),
        match color {
            PlayerColor::Red => piece.red_material.clone(),
            PlayerColor::Blue => piece.blue_material.clone(),
        },
        (x, height, y),
    );

    if let Some(winner) = winner::check_winner(&board.0, (x, y, height)) {
        game_winner.0 = Some(winner);
        info!("{} wins!", player_name(winner));
    } else {
        turn.change();
    }

    true
}

fn player_name(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Red => "Red",
        PlayerColor::Blue => "Blue",
    }
}

fn spawn_piece(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: (usize, usize, usize),
) {
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(
            position.0 as f32 * constants::SPACE + constants::OFFSET,
            (2.0 * position.1 as f32 + 1.0) * constants::PIECE_RING_RADIUS,
            position.2 as f32 * constants::SPACE + constants::OFFSET,
        ),
        Pickable::IGNORE,
    ));
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>()
            .init_resource::<PlayerTurn>()
            .init_resource::<Winner>()
            .init_resource::<BoardMaterial>()
            .init_resource::<RodMaterials>()
            .init_resource::<PieceMaterialsAndMeshes>()
            .add_plugins(MeshPickingPlugin)
            .add_systems(Startup, create_board)
            .add_systems(Update, apply_network_moves.after(NetworkReceiveSet));
    }
}
