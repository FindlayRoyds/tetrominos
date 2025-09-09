use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use strum::IntoEnumIterator;

pub mod placed_tile;
pub mod tetromino_data;

use crate::{
    board::{
        placed_tile::PlacedTile,
        tetromino_data::{
            TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_wall_kicks,
        },
    },
    input::{Action, get_board_input_map},
    tiles::{Tile, Tilemap, spawn_tile},
    try_unwrap,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                clear_tiles,
                apply_shift,
                apply_auto_shift,
                apply_soft_drop,
                apply_hard_drop,
                apply_gravity,
                apply_rotation,
                apply_sub_tile_offset,
                apply_placement,
                clear_lines,
                spawn_tiles,
            )
                .chain()
                .in_set(BoardUpdates),),
        );
    }
}

#[derive(Component)]
pub struct TetrominoTile;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct BoardUpdates;

#[derive(Component)]
pub struct Board {
    kind: TetrominoKind,
    pos: IVec2,
    rotation: TetrominoRotation,
    sub_tile_offset: Vec2,
    lock_delay: i32,
    auto_shift_delay: i32,
}

impl Board {
    pub fn new(kind: TetrominoKind, pos: IVec2) -> Self {
        Self {
            kind,
            pos,
            rotation: 0,
            sub_tile_offset: vec2(0.0, 0.0),
            lock_delay: 80,
            auto_shift_delay: 10,
        }
    }

    pub fn spawn_next(
        &mut self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    ) {
        let kind_variants: Vec<TetrominoKind> = TetrominoKind::iter().collect();
        self.kind = kind_variants[fastrand::usize(..kind_variants.len())];
        self.pos = ivec2(4, tilemap.size.y as i32 + 1);
        self.rotation = 0;
        self.lock_delay = 80;
        if !self.can_place(self_entity, tilemap, placed_tiles, self.pos, self.rotation) {
            bevy::log::error!("Attempted to spawn tetromino at invalid position");
        }
    }

    pub fn can_place(
        &self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
        new_pos: IVec2,
        new_rotation: TetrominoRotation,
    ) -> bool {
        let shape = get_tetromino_shape(self.kind, new_rotation);
        for offset in shape.iter() {
            let pos = new_pos + offset;
            if pos.x < 0
                || pos.x >= tilemap.size.x as i32
                || pos.y < 0
                || tilemap.get_tile(self_entity, pos, placed_tiles).is_some()
            {
                return false;
            }
        }

        true
    }

    pub fn place(
        &mut self,
        commands: &mut Commands,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
        asset_server: &Res<AssetServer>,
    ) {
        if self.can_place(self_entity, tilemap, placed_tiles, self.pos, self.rotation) {
            for offset in get_tetromino_shape(self.kind, self.rotation) {
                let pos = self.pos + offset;
                spawn_tile(commands, pos, self_entity, true, asset_server);
            }
        } else {
            bevy::log::error!("Failed to place tetromino at {:?}", self.pos);
        }
        self.spawn_next(self_entity, tilemap, placed_tiles);
    }
}

pub fn spawn_board(
    commands: &mut Commands,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rec_size = (size * tile_size).as_vec2();
    let tilemap = Tilemap { size, tile_size };
    let mut board = Board::new(TetrominoKind::L, ivec2(4, size.y as i32 + 1));

    let entity = commands
        .spawn((
            Name::new("Board"),
            Mesh2d(meshes.add(Rectangle::new(rec_size.x, rec_size.y))),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Transform::from_scale(vec3(4.0, 4.0, 4.0)),
            get_board_input_map(),
        ))
        .id();

    board.spawn_next(entity, &tilemap, placed_tiles);
    commands.entity(entity).insert(board).insert(tilemap);
}

fn clear_lines(
    mut commands: Commands,
    mut boards: Query<(Entity, &Tilemap), With<Board>>,
    mut tile_queries: ParamSet<(Query<(Entity, &Tile)>, Query<(Entity, &mut Tile)>)>,
) {
    for (board_entity, tilemap) in boards.iter_mut() {
        let mut num_cleared_lines = 0;

        for y in 0..tilemap.size.y as i32 {
            let mut tile_entities: Vec<(Entity, IVec2)> = vec![];
            for x in 0..tilemap.size.x as i32 {
                if let Some(tile_entity) =
                    tilemap.get_tile(board_entity, ivec2(x, y), tile_queries.p0())
                {
                    tile_entities.push((tile_entity, ivec2(x, y)));
                }
            }

            if tile_entities.len() == tilemap.size.x as usize {
                num_cleared_lines += 1;
                for (tile_entity, _) in tile_entities {
                    if let Ok(mut tile_commands) = commands.get_entity(tile_entity) {
                        tile_commands.despawn();
                    }
                }
            } else if num_cleared_lines > 0 {
                for (tile_entity, _) in tile_entities {
                    if let Ok((_, mut tile)) = tile_queries.p1().get_mut(tile_entity) {
                        tile.pos -= ivec2(0, num_cleared_lines);
                    }
                }
            }
        }
    }
}

fn clear_tiles(mut commands: Commands, tiles: Query<Entity, (With<Tile>, With<TetrominoTile>)>) {
    for tile_entity in tiles.iter() {
        try_unwrap!(commands.get_entity(tile_entity), "no entity in clear tiles").despawn();
    }
}

fn spawn_tiles(
    mut commands: Commands,
    boards: Query<(Entity, &Board)>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, board) in boards.iter() {
        for offset in get_tetromino_shape(board.kind, board.rotation) {
            let pos = board.pos + offset;
            let tile_entity = spawn_tile(&mut commands, pos, board_entity, false, &asset_server);
            try_unwrap!(commands.get_entity(tile_entity), "no entity in spawn tile")
                .insert(TetrominoTile);
        }
    }
}

fn apply_gravity(
    mut boards: Query<(Entity, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let new_pos = board.pos - ivec2(0, 1);
        if board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
            board.sub_tile_offset.y -= 0.05;
        }
    }
}

fn apply_shift(mut boards: Query<(&ActionState<Action>, &mut Board)>) {
    for (action_state, mut board) in boards.iter_mut() {
        if action_state.just_pressed(&Action::ShiftLeft) {
            board.sub_tile_offset.x -= 1.0;
        } else if action_state.just_pressed(&Action::ShiftRight) {
            board.sub_tile_offset.x += 1.0;
        }
    }
}

fn apply_auto_shift(mut boards: Query<(&ActionState<Action>, &mut Board)>) {
    for (action_state, mut board) in boards.iter_mut() {
        let shift = if action_state.pressed(&Action::ShiftLeft) {
            -1
        } else if action_state.pressed(&Action::ShiftRight) {
            1
        } else {
            0
        };

        if shift == 0 {
            board.auto_shift_delay = 10;
            continue;
        }
        if board.auto_shift_delay > 0 {
            board.auto_shift_delay -= 1;
            continue;
        }

        board.sub_tile_offset.x += 0.25 * shift as f32;
    }
}

fn apply_soft_drop(mut boards: Query<(&ActionState<Action>, &mut Board)>) {
    for (action_state, mut board) in boards.iter_mut() {
        if action_state.pressed(&Action::SoftDrop) {
            board.sub_tile_offset.y -= 0.25;
        }
    }
}

fn apply_hard_drop(
    mut commands: Commands,
    mut boards: Query<(Entity, &ActionState<Action>, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, action_state, mut board, tilemap) in boards.iter_mut() {
        if action_state.just_pressed(&Action::HardDrop) {
            for y_pos in 0..=board.pos.y {
                let new_pos = ivec2(board.pos.x, y_pos);
                if board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
                    board.pos = new_pos;
                    board.place(
                        &mut commands,
                        board_entity,
                        tilemap,
                        placed_tiles,
                        &asset_server,
                    );
                    break;
                }
            }
        }
    }
}

pub fn apply_rotation(
    mut boards: Query<(Entity, &ActionState<Action>, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
) {
    for (board_entity, action_state, mut board, tilemap) in boards.iter_mut() {
        let new_rotation = if action_state.just_pressed(&Action::RotateRight) {
            board.rotation + 1
        } else if action_state.just_pressed(&Action::RotateLeft) {
            board.rotation - 1
        } else {
            continue;
        };

        let offsets = get_tetromino_wall_kicks(board.rotation, new_rotation, board.kind);
        for offset in offsets.iter() {
            let new_pos = board.pos + offset;
            if board.can_place(board_entity, tilemap, placed_tiles, new_pos, new_rotation) {
                board.pos = new_pos;
                board.rotation = new_rotation;
                return;
            }
        }
        bevy::log::error!("All wall kicks failed!");
    }
}

fn apply_sub_tile_offset(
    mut boards: Query<(Entity, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
) {
    fn get_range(value: i32) -> Vec<i32> {
        if value.is_positive() {
            (1..=value).collect()
        } else {
            (value..0).rev().collect()
        }
    }

    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        board.sub_tile_offset.x = board.sub_tile_offset.x.clamp(-1.0, 1.0);
        let total_offset = board.sub_tile_offset.floor().as_ivec2();
        board.sub_tile_offset -= total_offset.as_vec2();

        for (axis, dir) in [ivec2(1, 0), ivec2(0, 1)].iter().enumerate() {
            for offset in get_range(total_offset[axis]) {
                let new_pos = board.pos + *dir * offset;
                if !board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
                    board.sub_tile_offset[axis] = 0.0;
                    break;
                }
                board.pos[axis] = new_pos[axis];
            }
        }
    }
}

fn apply_placement(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let new_pos = board.pos - ivec2(0, 1);
        if board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
            return;
        }

        board.lock_delay -= 1;
        if board.lock_delay < 0 {
            board.place(
                &mut commands,
                board_entity,
                tilemap,
                placed_tiles,
                &asset_server,
            );
        }
    }
}
