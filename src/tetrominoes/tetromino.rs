use bevy::prelude::*;

use crate::{
    board::Board,
    tetrominoes::{
        TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_wall_kicks,
    },
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
                apply_shift,
                apply_auto_shift,
                apply_soft_drop,
                apply_hard_drop,
                apply_gravity,
                apply_sub_tile_offset,
                apply_placement,
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
    pub rotation: i32, // 0..4
    // pub vertical_offset: f32, // Sub block offset
    pub sub_tile_offset: Vec2,
    pub lock_delay: i32, // Num frames of delay left until being placed

    pub board_entity: Entity,
}

impl Tetromino {
    pub fn new(kind: TetrominoKind, pos: IVec2, lock_delay: i32, board_entity: Entity) -> Self {
        Self {
            kind,
            pos,
            rotation: 0,
            sub_tile_offset: vec2(0.0, 0.0),
            lock_delay,

            board_entity,
        }
    }
}

#[derive(Component)]
pub struct TetrominoTile;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TetrominoUpdates;

pub fn rotate_tetromino(tetromino: &mut Tetromino, board: &Board, rotation: TetrominoRotation) {
    let offsets = get_tetromino_wall_kicks(tetromino.rotation, rotation, tetromino.kind);
    for offset in offsets.iter() {
        let new_pos = tetromino.pos + offset;
        if board.can_place(tetromino.kind, rotation, new_pos) {
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
    if board.can_place(tetromino.kind, tetromino.rotation, tetromino.pos) {
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
        if board.can_place(tetromino.kind, tetromino.rotation, new_pos) {
            tetromino.sub_tile_offset.y -= 0.05;
        }
    }
}

fn apply_shift(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board, auto shift");

        tetromino.sub_tile_offset.x += board.shift as f32;
    }
}

fn apply_auto_shift(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board, auto shift");

        tetromino.sub_tile_offset.x += 0.25 * board.auto_shift as f32;
    }
}

fn apply_soft_drop(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board, auto shift");

        if board.soft_drop {
            tetromino.sub_tile_offset.y -= 0.25;
        }
    }
}

fn apply_hard_drop(
    mut commands: Commands,
    mut tetrominoes: Query<(Entity, &mut Tetromino)>,
    boards: Query<&Board>,
    asset_server: Res<AssetServer>,
) {
    for (tetromino_entity, mut tetromino) in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board, auto shift");

        if board.hard_drop {
            for y_pos in 0..tetromino.pos.y {
                let new_pos = ivec2(tetromino.pos.x, y_pos);
                if board.can_place(tetromino.kind, tetromino.rotation, new_pos) {
                    tetromino.pos = new_pos;
                    place_tetromino(
                        &mut commands,
                        &tetromino,
                        tetromino_entity,
                        board,
                        &asset_server,
                    );
                    break;
                }
            }
        }
    }
}

fn apply_sub_tile_offset(mut tetrominoes: Query<&mut Tetromino>, boards: Query<&Board>) {
    fn get_range(value: i32) -> Vec<i32> {
        if value.is_positive() {
            (1..=value).collect()
        } else {
            (value..0).rev().collect()
        }
    }

    for mut tetromino in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "no board, up positions");

        tetromino.sub_tile_offset.x = tetromino.sub_tile_offset.x.clamp(-1.0, 1.0);
        let total_offset = tetromino.sub_tile_offset.floor().as_ivec2();
        tetromino.sub_tile_offset -= total_offset.as_vec2();

        for (axis, dir) in [ivec2(1, 0), ivec2(0, 1)].iter().enumerate() {
            for offset in get_range(total_offset[axis]) {
                let new_pos = tetromino.pos + *dir * offset;
                if !board.can_place(tetromino.kind, tetromino.rotation, new_pos) {
                    tetromino.sub_tile_offset[axis] = 0.0;
                    break;
                }
                tetromino.pos[axis] = new_pos[axis];
            }
        }
    }
}

fn apply_placement(
    mut commands: Commands,
    mut tetrominoes: Query<(Entity, &mut Tetromino)>,
    boards: Query<&Board>,
    asset_server: Res<AssetServer>,
) {
    for (tetromino_entity, mut tetromino) in tetrominoes.iter_mut() {
        let board = try_unwrap!(boards.get(tetromino.board_entity), "No board in place");

        let new_pos = tetromino.pos - ivec2(0, 1);
        if board.can_place(tetromino.kind, tetromino.rotation, new_pos) {
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
