use bevy::prelude::*;
use rand::{self, RngCore, SeedableRng};
use rand_pcg::Pcg32;

mod board;
mod input;
mod rng;
mod tiles;
mod warnings;

use crate::{
    board::{
        BoardPlugin,
        placed_tile::PlacedTile,
        spawn_board,
        tile_assets::{TileImages, TileOutlineImages},
    },
    input::InputPlugin,
    rng::RandomSource,
    tiles::{Tile, TilePlugin},
};

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()),))
        .add_plugins((TilePlugin, BoardPlugin, InputPlugin))
        .add_systems(Startup, setup)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .run()
}

fn setup(
    mut commands: Commands,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
) {
    commands.spawn(Camera2d);

    let random_seed = rand::thread_rng().next_u64();
    let mut rng = Pcg32::seed_from_u64(random_seed);
    commands.insert_resource(RandomSource(rng.clone()));

    spawn_board(
        &mut commands,
        placed_tiles,
        uvec2(10, 20),
        uvec2(8, 8),
        &mut meshes,
        &mut materials,
        tile_images,
        tile_outline_images,
        &mut rng,
    );
}
