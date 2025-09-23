use bevy::prelude::*;

mod board;
mod input;
mod tiles;
mod warnings;

use crate::{
    board::{BoardPlugin, BoardUpdates, placed_tile::PlacedTile, spawn_board},
    input::InputPlugin,
    tiles::{Tile, TilePlugin, TileVisuals},
};

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((TilePlugin, BoardPlugin, InputPlugin))
        .add_systems(Startup, setup)
        .configure_sets(Update, (BoardUpdates, TileVisuals).chain())
        .run()
}

fn setup(
    mut commands: Commands,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
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
