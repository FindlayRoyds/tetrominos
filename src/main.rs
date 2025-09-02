use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use fastrand;

use crate::{
    board::Board,
    tetromino::{
        Tetromino, TetrominoKind, TetrominoUpdates, is_tetromino_pos_valid, rotate_tetromino,
    },
    tile::{TileUpdates, TileVisuals},
};

mod board;
mod tetromino;
mod tile;

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((tile::TilePlugin, tetromino::TetrominoPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (handle_keypress).in_set(TileUpdates),
                (rotate_tetrominos, move_tetrominos).before(TetrominoUpdates),
            ),
        )
        .configure_sets(Update, (TetrominoUpdates, TileUpdates, TileVisuals).chain())
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 70.0, 0.0).with_scale(Vec3::splat(0.3)),
    ));

    commands.spawn((Name::new("Board"), Board::new(uvec2(10, 20))));
}

fn handle_keypress(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    boards: Query<(Entity, &Board)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let (board_entity, board) = boards
            .single()
            .expect("Expected one board when spawning tetromino");

        let kind = match fastrand::i32(0..7) {
            0 => TetrominoKind::I,
            1 => TetrominoKind::J,
            2 => TetrominoKind::L,
            3 => TetrominoKind::O,
            4 => TetrominoKind::S,
            5 => TetrominoKind::T,
            _ => TetrominoKind::Z,
        };
        let pos = ivec2(4, board.size.y as i32);
        if !is_tetromino_pos_valid(kind, 0, pos, board) {
            bevy::log::warn!("Attempted to spawn tetromino at invalid position");
            return;
        }
        commands.entity(board_entity).with_children(|parent| {
            parent.spawn((
                Name::new("Tetromino"),
                Tetromino::new(kind, pos, 80, board_entity),
            ));
        });
    }
}

fn move_tetrominos(
    mut tetrominos: Query<&mut Tetromino>,
    boards: Query<&Board>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut movement = IVec2::ZERO;
    if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        movement.x -= 1;
    }
    if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        movement.x += 1;
    }

    for mut tetromino in tetrominos.iter_mut() {
        let board = boards.get(tetromino.board_entity).expect("Board not found");

        let new_pos = tetromino.pos + movement;
        if is_tetromino_pos_valid(tetromino.kind, tetromino.rotation, new_pos, board) {
            tetromino.pos = new_pos;
        }
    }
}

fn rotate_tetrominos(
    mut tetrominos: Query<&mut Tetromino>,
    boards: Query<&Board>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp) {
        for mut tetromino in tetrominos.iter_mut() {
            let board = boards.get(tetromino.board_entity).expect("Board not found");
            let new_rotation = (tetromino.rotation + 1) % 4;
            rotate_tetromino(&mut tetromino, board, new_rotation);
        }
    }
}
