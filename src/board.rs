use bevy::prelude::*;
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
    tetronimo_kind: TetrominoKind,
    tetromino_pos: IVec2,
    tetromino_rotation: TetrominoRotation,
    tetromino_sub_tile_offset: Vec2,
    tetromino_lock_delay: i32,

    // Inputs
    pub shift_input: i32,
    pub auto_shift_input: i32,
    pub auto_shift_delay_input: i32,
    pub soft_drop_input: bool,
    pub hard_drop_input: bool,
    pub rotate_input: i32,
}

impl Board {
    pub fn new(kind: TetrominoKind, pos: IVec2) -> Self {
        Self {
            tetronimo_kind: kind,
            tetromino_pos: pos,
            tetromino_rotation: 0,
            tetromino_sub_tile_offset: vec2(0.0, 0.0),
            tetromino_lock_delay: 80,

            shift_input: 0,
            auto_shift_delay_input: 0,
            auto_shift_input: 0,
            soft_drop_input: false,
            hard_drop_input: false,
            rotate_input: 0,
        }
    }

    pub fn spawn_next(
        &mut self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    ) {
        let kind_variants: Vec<TetrominoKind> = TetrominoKind::iter().collect();
        self.tetronimo_kind = kind_variants[fastrand::usize(..kind_variants.len())];
        self.tetromino_pos = ivec2(4, tilemap.size.y as i32 + 1);
        self.tetromino_rotation = 0;
        self.tetromino_lock_delay = 80;
        if !self.can_place(
            self_entity,
            tilemap,
            placed_tiles,
            self.tetromino_pos,
            self.tetromino_rotation,
        ) {
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
        let shape = get_tetromino_shape(self.tetronimo_kind, new_rotation);
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
        if self.can_place(
            self_entity,
            tilemap,
            placed_tiles,
            self.tetromino_pos,
            self.tetromino_rotation,
        ) {
            for offset in get_tetromino_shape(self.tetronimo_kind, self.tetromino_rotation) {
                let pos = self.tetromino_pos + offset;
                spawn_tile(commands, pos, self_entity, true, asset_server);
            }
        } else {
            bevy::log::error!("Failed to place tetromino at {:?}", self.tetromino_pos);
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
        for offset in get_tetromino_shape(board.tetronimo_kind, board.tetromino_rotation) {
            let pos = board.tetromino_pos + offset;
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
        let new_pos = board.tetromino_pos - ivec2(0, 1);
        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            new_pos,
            board.tetromino_rotation,
        ) {
            board.tetromino_sub_tile_offset.y -= 0.05;
        }
    }
}

fn apply_shift(mut boards: Query<&mut Board>) {
    for mut board in boards.iter_mut() {
        board.tetromino_sub_tile_offset.x += board.shift_input as f32;
    }
}

fn apply_auto_shift(mut boards: Query<&mut Board>) {
    for mut board in boards.iter_mut() {
        board.tetromino_sub_tile_offset.x += 0.25 * board.auto_shift_input as f32;
    }
}

fn apply_soft_drop(mut boards: Query<&mut Board>) {
    for mut board in boards.iter_mut() {
        if board.soft_drop_input {
            board.tetromino_sub_tile_offset.y -= 0.25;
        }
    }
}

fn apply_hard_drop(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    asset_server: Res<AssetServer>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        if board.hard_drop_input {
            for y_pos in 0..=board.tetromino_pos.y {
                let new_pos = ivec2(board.tetromino_pos.x, y_pos);
                if board.can_place(
                    board_entity,
                    tilemap,
                    placed_tiles,
                    new_pos,
                    board.tetromino_rotation,
                ) {
                    board.tetromino_pos = new_pos;
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
    mut boards: Query<(Entity, &mut Board, &Tilemap)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
) {
    for (board_entity, mut board, tilemap) in boards.iter_mut() {
        let new_rotation = board.tetromino_rotation + board.rotate_input;
        let offsets =
            get_tetromino_wall_kicks(board.tetromino_rotation, new_rotation, board.tetronimo_kind);
        for offset in offsets.iter() {
            let new_pos = board.tetromino_pos + offset;
            if board.can_place(board_entity, tilemap, placed_tiles, new_pos, new_rotation) {
                board.tetromino_pos = new_pos;
                board.tetromino_rotation = new_rotation;
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
        board.tetromino_sub_tile_offset.x = board.tetromino_sub_tile_offset.x.clamp(-1.0, 1.0);
        let total_offset = board.tetromino_sub_tile_offset.floor().as_ivec2();
        board.tetromino_sub_tile_offset -= total_offset.as_vec2();

        for (axis, dir) in [ivec2(1, 0), ivec2(0, 1)].iter().enumerate() {
            for offset in get_range(total_offset[axis]) {
                let new_pos = board.tetromino_pos + *dir * offset;
                if !board.can_place(
                    board_entity,
                    tilemap,
                    placed_tiles,
                    new_pos,
                    board.tetromino_rotation,
                ) {
                    board.tetromino_sub_tile_offset[axis] = 0.0;
                    break;
                }
                board.tetromino_pos[axis] = new_pos[axis];
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
        let new_pos = board.tetromino_pos - ivec2(0, 1);
        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            new_pos,
            board.tetromino_rotation,
        ) {
            return;
        }

        board.tetromino_lock_delay -= 1;
        if board.tetromino_lock_delay < 0 {
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
