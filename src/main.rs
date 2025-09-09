use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod board;
mod tiles;
mod warnings;

use crate::{
    board::{Board, BoardPlugin, BoardUpdates, placed_tile::PlacedTile, spawn_board},
    tiles::{Tile, TilePlugin, TileVisuals},
};

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((TilePlugin, BoardPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input_rotate, input_shift, input_soft_drop, input_hard_drop).before(InputSet),
        )
        .configure_sets(Update, (InputSet, BoardUpdates, TileVisuals).chain())
        .run()
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct InputSet;

fn setup(
    mut commands: Commands,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    spawn_board(
        &mut commands,
        placed_tiles,
        uvec2(10, 20),
        uvec2(8, 8),
        &mut meshes,
        &mut materials,
    );
}

fn input_rotate(mut boards: Query<&mut Board>, keyboard: Res<ButtonInput<KeyCode>>) {
    let rotation =
        if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp) {
            1
        } else {
            0
        };

    for mut board in boards.iter_mut() {
        board.rotate_input = rotation;
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
        board.shift_input = shift;

        if auto_shift == 0 {
            board.auto_shift_delay_input = 10;
            board.auto_shift_input = 0;
            continue;
        }
        if board.auto_shift_delay_input > 0 {
            board.auto_shift_delay_input -= 1;
            continue;
        }
        board.auto_shift_input = auto_shift;
    }
}

fn input_soft_drop(mut boards: Query<&mut Board>, keyboard: Res<ButtonInput<KeyCode>>) {
    let soft_drop = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    for mut board in boards.iter_mut() {
        board.soft_drop_input = soft_drop;
    }
}

fn input_hard_drop(mut boards: Query<&mut Board>, keyboard: Res<ButtonInput<KeyCode>>) {
    let hard_drop = keyboard.just_pressed(KeyCode::Space);
    for mut board in boards.iter_mut() {
        board.hard_drop_input = hard_drop;
    }
}
