use bevy::prelude::*;

use crate::{
    board::Board,
    tetromino::{TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_wall_kicks},
    tile::{Tile, spawn_tile},
    try_unwrap,
};

pub struct TetrominoPlugin;

impl Plugin for TetrominoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                clear_tiles,
                apply_gravity,
                update_positions,
                place,
                spawn_tiles,
            )
                .chain()
                .in_set(TetrominoUpdates),
        );
    }
}

#[derive(Component)]
pub struct Tetromino {
    pub kind: TetrominoKind,
    pub pos: IVec2,
    pub rotation: i32,        // 0..4
    pub vertical_offset: f32, // Sub block offset
    pub lock_delay: i32,      // Num frames of delay left until being placed

    pub board_entity: Entity,
}

impl Tetromino {
    pub fn new(kind: TetrominoKind, pos: IVec2, lock_delay: i32, board_entity: Entity) -> Self {
        Self {
            kind,
            pos,
            rotation: 0,
            vertical_offset: 0.0,
            lock_delay,

            board_entity,
        }
    }
}

#[derive(Component)]
pub struct TetrominoTile;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TetrominoUpdates;

pub fn is_tetromino_pos_valid(
    kind: TetrominoKind,
    rotation: TetrominoRotation,
    new_pos: IVec2,
    board: &Board,
) -> bool {
    let shape = get_tetromino_shape(kind, rotation);
    for offset in shape.iter() {
        let pos = new_pos + offset;
        if pos.x < 0 || pos.x >= board.size.x as i32 || pos.y < 0 || board.get_tile(pos).is_some() {
            return false;
        }
    }

    return true;
}

pub fn rotate_tetromino(tetromino: &mut Tetromino, board: &Board, rotation: TetrominoRotation) {
    let offsets = get_tetromino_wall_kicks(tetromino.rotation, rotation, tetromino.kind);
    for offset in offsets.iter() {
        let new_pos = tetromino.pos + offset;
        if is_tetromino_pos_valid(tetromino.kind, rotation, new_pos, board) {
            tetromino.pos = new_pos;
            tetromino.rotation = rotation;
            return;
        }
    }
}

fn place_tetromino(
    commands: &mut Commands,
    tetromino: &Tetromino,
    tetromino_entity: Entity,
    board: &Board,
    asset_server: &Res<AssetServer>,
) {
    if is_tetromino_pos_valid(tetromino.kind, tetromino.rotation, tetromino.pos, board) {
        for offset in get_tetromino_shape(tetromino.kind, tetromino.rotation) {
            let pos = tetromino.pos + offset;
            spawn_tile(commands, pos, tetromino.board_entity, true, asset_server);
        }
    } else {
        bevy::log::error!("Failed to place tetromino at {:?}", tetromino.pos);
    }

    commands.entity(tetromino_entity).despawn();
}

// ========== Systems ==========

fn clear_tiles(mut commands: Commands, tiles: Query<Entity, (With<Tile>, With<TetrominoTile>)>) {
    for tile_entity in tiles.iter() {
        try_unwrap!(commands.get_entity(tile_entity), "to entity in clear tiles").despawn();
    }
}

fn spawn_tiles(
    mut commands: Commands,
    tetrominoes: Query<&Tetromino>,
    asset_server: Res<AssetServer>,
) {
    for tetromino in tetrominoes.iter() {
        let board_entity = tetromino.board_entity;
        for offset in get_tetromino_shape(tetromino.kind, tetromino.rotation) {
            let pos = tetromino.pos + offset;
            let tile_entity = spawn_tile(&mut commands, pos, board_entity, false, &asset_server);
            try_unwrap!(commands.get_entity(tile_entity), "no entity in spawn tile")
                .insert(TetrominoTile);
        }
    }
}

fn apply_gravity(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board in fn gravity");

        let new_pos = tetromino.pos - ivec2(0, 1);
        if is_tetromino_pos_valid(tetromino.kind, tetromino.rotation, new_pos, board) {
            tetromino.vertical_offset -= 0.1;
        }
    }
}

fn update_positions(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "no board, up positions");

        let total_offset = tetromino.vertical_offset.floor() as i32; // Negative number
        let mut final_pos = tetromino.pos;
        for offset in (total_offset..0).rev() {
            let new_pos = tetromino.pos + ivec2(0, offset);
            if is_tetromino_pos_valid(tetromino.kind, tetromino.rotation, new_pos, board) {
                final_pos = new_pos;
            }
        }

        tetromino.pos = final_pos;
        tetromino.vertical_offset -= total_offset as f32;
    }
}

fn place(
    mut commands: Commands,
    mut tetrominoes: Query<(Entity, &mut Tetromino)>,
    boards: Query<&Board>,
    asset_server: Res<AssetServer>,
) {
    for (tetromino_entity, mut tetromino) in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board in fn place");

        let new_pos = tetromino.pos - ivec2(0, 1);
        if is_tetromino_pos_valid(tetromino.kind, tetromino.rotation, new_pos, board) {
            return;
        }

        tetromino.lock_delay -= 1;
        if tetromino.lock_delay < 0 {
            place_tetromino(
                &mut commands,
                &tetromino,
                tetromino_entity,
                board,
                &asset_server,
            );
        }
    }
}
