use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use fastrand;

use crate::tile::{Board, SpawnTile, Tile};

mod tile;

fn main() -> AppExit {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .add_plugins(tile::TilePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_keypress, update_tile_positions))
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 70.0, 0.0).with_scale(Vec3::splat(0.3)),
    ));

    let board = commands
        .spawn((Name::new("Board"), Board::new(UVec2::new(10, 20))))
        .id();

    commands.queue(SpawnTile {
        pos: IVec2::new(0, 10),
        board,
    });
    commands.queue(SpawnTile {
        pos: IVec2::new(0, 11),
        board,
    });
}

fn update_tile_positions(mut tiles: Query<&mut Tile>, mut boards: Query<&mut Board>) {
    for mut tile in tiles.iter_mut() {
        let board = boards.get_mut(tile.board).expect("Board not found");

        let new_pos = tile.get_pos() - IVec2::new(0, 1);
        if board.get_tile(new_pos).is_none() && board.is_in_bounds(new_pos) {
            tile.set_pos(new_pos, &mut boards);
        }
    }
}

fn handle_keypress(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    board_entities: Query<(Entity, &Board)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let (board_entity, board) = board_entities.single().expect("No board found");
        let pos = IVec2::new(fastrand::i32(0..board.size.x as i32), 19);
        if board.get_tile(pos).is_some() {
            return;
        }
        commands.queue(SpawnTile {
            pos,
            board: board_entity,
        });
    }
}
