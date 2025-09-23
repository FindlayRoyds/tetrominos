use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use strum::IntoEnumIterator;

pub mod placed_tile;
mod tetromino_data;

use crate::{
    board::{
        placed_tile::PlacedTile,
        tetromino_data::{
            TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_wall_kicks,
        },
    },
    input::{Action, get_board_input_map},
    tiles::{Tile, Tilemap},
    try_unwrap,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                apply_shift,
                apply_auto_shift,
                apply_soft_drop,
                apply_hard_drop,
                apply_gravity,
                apply_rotation,
                apply_movement,
                apply_collisions,
                apply_placement,
                clear_lines,
                clear_tetromino_tiles,
                spawn_tetromino_tiles,
            )
                .chain()
                .in_set(BoardUpdates),),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct BoardUpdates;

#[derive(Component)]
pub struct TetrominoTile;

#[derive(Component)]
pub struct SkipUpdate;

#[derive(Component)]
pub struct Board {
    kind: TetrominoKind,
    pos: Vec2,
    movement: Vec2,
    rotation: TetrominoRotation,
    lock_delay: i32,
    auto_shift_delay: i32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            kind: TetrominoKind::L,
            pos: vec2(0.0, 0.0),
            movement: vec2(0.0, 0.0),
            rotation: 0,
            lock_delay: 80,
            auto_shift_delay: 10,
        }
    }
}

impl Board {
    pub fn spawn_next(
        &mut self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
    ) {
        let kind_variants: Vec<TetrominoKind> = TetrominoKind::iter().collect();
        self.kind = kind_variants[fastrand::usize(..kind_variants.len())];
        self.pos = vec2(4.0, tilemap.size.y as f32 - 1.0);
        self.rotation = 0;
        self.lock_delay = 80;
        if !self.can_place(
            self_entity,
            tilemap,
            placed_tiles,
            self.get_snapped_pos(),
            self.rotation,
        ) {
            bevy::log::error!("Attempted to spawn tetromino at invalid position");
        }
    }

    pub fn can_place(
        &self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
        new_pos: IVec2,
        new_rotation: TetrominoRotation,
    ) -> bool {
        let shape = get_tetromino_shape(self.kind, new_rotation);
        for offset in shape.iter() {
            let pos = new_pos + offset;
            if pos.x < 0
                || pos.x >= tilemap.size.x as i32
                || pos.y < 0
                || tilemap.is_tile(self_entity, pos.as_vec2(), placed_tiles)
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
        placed_tiles: Query<&Tile, With<PlacedTile>>,
        asset_server: &Res<AssetServer>,
    ) {
        if self.can_place(
            self_entity,
            tilemap,
            placed_tiles,
            self.get_snapped_pos(),
            self.rotation,
        ) {
            for offset in get_tetromino_shape(self.kind, self.rotation) {
                let pos = self.get_snapped_pos() + offset;
                commands.spawn((
                    Name::new("PlacedTile"),
                    Tile {
                        pos: pos.as_vec2(),
                        tilemap: self_entity,
                    },
                    PlacedTile,
                    ChildOf(self_entity),
                    Sprite::from_image(asset_server.load("tiles/tile.png")),
                ));
            }
        } else {
            bevy::log::error!("Failed to place tetromino at {:?}", self.pos);
        }
        self.spawn_next(self_entity, tilemap, placed_tiles);
    }

    fn get_snapped_pos(&self) -> IVec2 {
        snap_vec2(self.pos)
    }
}

fn snap_vec2(value: Vec2) -> IVec2 {
    value.round().as_ivec2()
}

// ========== Systems ==========

pub fn spawn_board(
    commands: &mut Commands,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rec_size = (size * tile_size).as_vec2();
    let tilemap = Tilemap { size, tile_size };
    let mut board = Board::default();

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
    mut boards: Query<(Entity, &Tilemap), (With<Board>, Without<SkipUpdate>)>,
    mut tile_queries: ParamSet<(
        Query<(Entity, &Tile), With<PlacedTile>>,
        Query<(Entity, &mut Tile), With<PlacedTile>>,
    )>,
) {
    for (board_entity, tilemap) in boards.iter_mut() {
        let mut num_cleared_lines = 0;

        for y in 0..tilemap.size.y as i32 {
            let mut clear_line = true;
            let mut tiles_to_clear: Vec<(Entity, IVec2)> = vec![];

            for x in 0..tilemap.size.x as i32 {
                let tile_entities =
                    tilemap.get_tiles(board_entity, ivec2(x, y).as_vec2(), tile_queries.p0());
                if !tile_entities.is_empty() {
                    for tile_entity in tile_entities {
                        tiles_to_clear.push((tile_entity, ivec2(x, y)));
                    }
                } else {
                    clear_line = false;
                }
            }

            if clear_line {
                num_cleared_lines += 1;
                for (tile_entity, _) in tiles_to_clear {
                    if let Ok(mut tile_commands) = commands.get_entity(tile_entity) {
                        tile_commands.despawn();
                    }
                }
            } else if num_cleared_lines > 0 {
                for (tile_entity, _) in tiles_to_clear {
                    if let Ok((_, mut tile)) = tile_queries.p1().get_mut(tile_entity) {
                        tile.pos -= ivec2(0, num_cleared_lines).as_vec2();
                    }
                }
            }
        }
    }
}

fn apply_gravity(mut boards: Query<&mut Board, Without<SkipUpdate>>) {
    for mut board in boards.iter_mut() {
        board.movement.y -= 0.05;
    }
}

fn apply_shift(mut boards: Query<(&ActionState<Action>, &mut Board), Without<SkipUpdate>>) {
    for (action_state, mut board) in boards.iter_mut() {
        if action_state.just_pressed(&Action::ShiftLeft) {
            board.movement.x -= 1.0;
        } else if action_state.just_pressed(&Action::ShiftRight) {
            board.movement.x += 1.0;
        }
    }
}

fn apply_auto_shift(mut boards: Query<(&ActionState<Action>, &mut Board), Without<SkipUpdate>>) {
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
            board.pos.x = board.get_snapped_pos().x as f32;
            continue;
        }
        if board.auto_shift_delay > 0 {
            board.auto_shift_delay -= 1;
            continue;
        }

        board.movement.x += 0.25 * shift as f32;
    }
}

fn apply_soft_drop(mut boards: Query<(&ActionState<Action>, &mut Board), Without<SkipUpdate>>) {
    for (action_state, mut board) in boards.iter_mut() {
        if action_state.pressed(&Action::SoftDrop) {
            board.movement.y -= 0.25;
        }
    }
}

fn apply_hard_drop(
    mut commands: Commands,
    mut boards: Query<(Entity, &ActionState<Action>, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, action_state, mut board, tilemap) in boards.iter_mut() {
        if action_state.just_pressed(&Action::HardDrop) {
            for y_pos in 0..=board.get_snapped_pos().y {
                let new_pos = ivec2(board.get_snapped_pos().x, y_pos);
                if board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
                    board.pos = new_pos.as_vec2();
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
    mut boards: Query<(Entity, &ActionState<Action>, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
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
            let new_pos = board.get_snapped_pos() + offset;
            if board.can_place(board_entity, tilemap, placed_tiles, new_pos, new_rotation) {
                board.pos += offset.as_vec2();
                board.rotation = new_rotation;
                return;
            }
        }
        bevy::log::warn!("All wall kicks failed!");
    }
}

fn apply_movement(
    mut boards: Query<(Entity, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
) {
    fn get_range(value: i32) -> Vec<i32> {
        if value.is_positive() {
            (0..=value).collect()
        } else {
            (value..=0).rev().collect()
        }
    }

    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let start = board.pos;
        let end = board.pos + board.movement;

        for (axis, dir) in [ivec2(1, 0), ivec2(0, 1)].iter().enumerate() {
            let mut final_pos = board.pos;

            let mut broke = false;
            for offset in get_range((snap_vec2(end) - snap_vec2(start))[axis]) {
                let new_pos = snap_vec2(board.pos) + dir * offset;
                if board.can_place(board_entity, tilemap, placed_tiles, new_pos, board.rotation) {
                    final_pos = board.pos + (dir * offset).as_vec2();
                } else {
                    broke = true;
                    break;
                }
            }
            if !broke {
                board.pos = board.pos + board.movement * dir.as_vec2();
            } else {
                board.pos = final_pos;
            }
        }

        board.movement = vec2(0.0, 0.0);
    }
}

fn apply_collisions(
    mut boards: Query<(Entity, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let snapped_pos = board.get_snapped_pos();
        for (axis, dir) in [vec2(1.0, 0.0), vec2(0.0, 1.0)].iter().enumerate() {
            let sub_tile_dir = dir.copysign(board.pos - snapped_pos.as_vec2()).as_ivec2();
            if !board.can_place(
                board_entity,
                tilemap,
                placed_tiles,
                snapped_pos + sub_tile_dir,
                board.rotation,
            ) {
                board.pos[axis] = snapped_pos[axis] as f32;
            }
        }
    }
}

fn apply_placement(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let pos_below = board.get_snapped_pos() - ivec2(0, 1);
        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            pos_below,
            board.rotation,
        ) {
            return; // Piece can still move down
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

fn clear_tetromino_tiles(
    mut commands: Commands,
    tiles: Query<Entity, (With<Tile>, With<TetrominoTile>)>,
) {
    for tile_entity in tiles.iter() {
        try_unwrap!(commands.get_entity(tile_entity), "no entity in clear tiles").despawn();
    }
}

fn spawn_tetromino_tiles(
    mut commands: Commands,
    boards: Query<(Entity, &Board)>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, board) in boards.iter() {
        for offset in get_tetromino_shape(board.kind, board.rotation) {
            let pos = (board.get_snapped_pos() + offset).as_vec2();
            // let pos = board.pos + offset.as_vec2();
            commands.spawn((
                Name::new("TetrominoTile"),
                Tile {
                    pos,
                    tilemap: board_entity,
                },
                TetrominoTile,
                ChildOf(board_entity),
                Sprite::from_image(asset_server.load("tiles/tile.png")),
            ));
            // commands.spawn((
            //     Name::new("TetrominoTile"),
            //     Tile {
            //         pos: board.pos + offset.as_vec2(),
            //         tilemap: board_entity,
            //     },
            //     TetrominoTile,
            //     ChildOf(board_entity),
            //     Sprite::from_image(asset_server.load("tiles/line_clear.png")),
            // ));
        }
    }
}
