use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    board::Board,
    tetromino::{Tetromino, TetrominoUpdates, rotate_tetromino},
    tile::{TileUpdates, TileVisuals},
};

mod board;
mod tetromino;
mod tile;
mod warnings;

use board::*;
use tetromino::*;
use tile::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((TilePlugin, TetrominoPlugin, BoardPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (rotate_tetrominoes, move_tetrominoes).before(TetrominoUpdates),
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
    commands.spawn((Camera2d::default(),));

    spawn_board(
        &mut commands,
        uvec2(10, 20),
        uvec2(8, 8),
        40,
        &mut meshes,
        &mut materials,
    );
}

fn move_tetrominoes(mut tetrominoes: Query<&mut Tetromino>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut movement = IVec2::ZERO;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        movement.x += 1;
    }

    for mut tetromino in tetrominoes.iter_mut() {
        if movement.x == 0 {
            tetromino.sub_tile_offset.x = 0.0;
            return;
        }
        tetromino.sub_tile_offset.x += movement.x as f32 * 0.125
    }
}

fn rotate_tetrominoes(
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
