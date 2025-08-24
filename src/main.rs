use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use fastrand;

use crate::{
    tetromino::{Tetromino, TetrominoUpdates, is_tetromino_pos_valid},
    tile::{Board, TileUpdates, TileVisuals},
};

mod tetromino;
mod tile;

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins((tile::TilePlugin, tetromino::TetrominoPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keypress.in_set(TileUpdates))
        .configure_sets(
            Update,
            (TetrominoUpdates, TileUpdates, TileVisuals).chain(),
        )
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 70.0, 0.0).with_scale(Vec3::splat(0.3)),
    ));

    commands.spawn((Name::new("Board"), Board::new(UVec2::new(10, 20))));
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

        let pos = IVec2::new(
            fastrand::i32(0..board.size.x as i32 - 1),
            board.size.y as i32,
        );
        let shape = vec![
            IVec2::new(0, 0),
            IVec2::new(0, 1),
            IVec2::new(0, 2),
            IVec2::new(1, 0),
        ];
        if !is_tetromino_pos_valid(shape.clone(), pos, board) {
            bevy::log::warn!("Attempted to spawn tetromino at invalid position");
            return;
        }
        commands.entity(board_entity).with_children(|parent| {
            parent.spawn((
                Name::new("Tetromino"),
                Tetromino::new(shape, pos, board_entity),
            ));
        });
    }
}
