use bevy::prelude::*;
use rand::{self, RngCore, SeedableRng};
use rand_pcg::Pcg32;

mod board;
mod input;
mod rng;
mod tiles;

use crate::{
    board::{BoardPlugin, SpawnNextTetromino, spawn_board},
    input::InputPlugin,
    rng::RandomSource,
    tiles::TilePlugin,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    spawn_next_messages: MessageWriter<SpawnNextTetromino>,
) {
    commands.spawn(Camera2d);

    let random_seed = rand::rng().next_u64();
    let mut rng = Pcg32::seed_from_u64(random_seed);
    commands.insert_resource(RandomSource(rng.clone()));

    spawn_board(
        &mut commands,
        uvec2(10, 20),
        uvec2(8, 8),
        &mut meshes,
        &mut materials,
        &mut rng,
        spawn_next_messages,
    );
}
