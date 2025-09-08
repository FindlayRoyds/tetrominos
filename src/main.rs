use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod board;
mod tetrominoes;
mod tile;
mod warnings;

use crate::{
    board::{Board, BoardPlugin, TetrominoSpawning, spawn_board},
    tetrominoes::{Tetromino, TetrominoPlugin, TetrominoUpdates, rotate_tetromino},
    tile::{TilePlugin, TileUpdates, TileVisuals},
};

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((TilePlugin, TetrominoPlugin, BoardPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input_rotate, input_shift, input_soft_drop).before(TetrominoUpdates),
        )
        .configure_sets(
            Update,
            (
                TetrominoSpawning,
                TetrominoUpdates,
                TileUpdates,
                TileVisuals,
            )
                .chain(),
        )
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2d,));

    spawn_board(
        &mut commands,
        uvec2(10, 20),
        uvec2(8, 8),
        40,
        &mut meshes,
        &mut materials,
    );
}

fn input_rotate(
    mut tetrominoes: Query<&mut Tetromino>,
    boards: Query<&Board>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp) {
        for mut tetromino in tetrominoes.iter_mut() {
            let board = try_unwrap!(boards.get(tetromino.board_entity), "No board in rotate");
            let new_rotation = (tetromino.rotation + 1) % 4;
            rotate_tetromino(&mut tetromino, board, new_rotation);
        }
    }
}

fn input_shift(mut boards: Query<&mut Board>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut auto_shift = 0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        auto_shift -= 1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        auto_shift += 1;
    }

    let mut shift = 0;
    if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        shift -= 1;
    }
    if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        shift += 1;
    }

    for mut board in boards.iter_mut() {
        board.shift = shift;

        if auto_shift == 0 {
            board.auto_shift_delay = 10;
            board.auto_shift = 0;
            continue;
        }
        if board.auto_shift_delay > 0 {
            board.auto_shift_delay -= 1;
            continue;
        }
        board.auto_shift = auto_shift;
    }
}

fn input_soft_drop(mut boards: Query<&mut Board>, keyboard: Res<ButtonInput<KeyCode>>) {
    let soft_drop = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    for mut board in boards.iter_mut() {
        board.soft_drop = soft_drop;
    }
}
