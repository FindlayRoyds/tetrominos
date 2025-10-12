use std::collections::VecDeque;

use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_shuffle_bag::ShuffleBag;
use leafwing_input_manager::prelude::ActionState;
use rand::Rng;
use strum::IntoEnumIterator;

mod board_config;
mod ghost_tile;
pub mod hold_display;
mod line_clear;
mod outline;
pub mod placed_tile;
mod tetromino_data;
mod tetromino_tile;
pub mod tile_assets;

use crate::{
    board::{
        board_config::BoardConfig,
        ghost_tile::{GhostTile, GhostTilePlugin, clear_ghost_tiles, spawn_ghost_tiles},
        hold_display::{HoldDisplay, HoldDisplayPlugin},
        line_clear::LineClearPlugin,
        placed_tile::PlacedTile,
        tetromino_data::{
            TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_wall_kicks,
        },
        tetromino_tile::{
            TetrominoTile, TetrominoTilePlugin, clear_tetromino_tiles, spawn_tetromino_tiles,
        },
        tile_assets::{TileAssets, TileImages, TileOutlineImages},
    },
    input::{Action, get_board_input_map},
    rng::RandomSource,
    tiles::{Tile, TileUpdateSystems, Tilemap},
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LineClearPlugin,
            TetrominoTilePlugin,
            GhostTilePlugin,
            TileAssets,
            HoldDisplayPlugin,
        ))
        .add_systems(
            FixedUpdate,
            (
                (
                    move_lines_down,
                    apply_hold,
                    apply_shift,
                    apply_auto_shift,
                    apply_soft_drop,
                    apply_hard_drop,
                    apply_gravity,
                    apply_rotation,
                    apply_movement,
                    apply_collisions,
                    apply_placement,
                )
                    .chain()
                    .in_set(BoardUpdateSystems),
                remove_skip_update.in_set(RemoveSkipUpdateSystems),
            ),
        )
        .configure_sets(
            FixedUpdate,
            (
                BoardUpdateSystems.before(TileUpdateSystems),
                RemoveSkipUpdateSystems,
                AddSkipUpdateSystems,
            )
                .chain(),
        )
        .add_message::<HoldPieceChanged>();
    }
}

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
struct BoardUpdateSystems;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
struct RemoveSkipUpdateSystems;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
struct AddSkipUpdateSystems;

#[derive(Component)]
pub struct SkipUpdate;

#[derive(Message)]
pub struct HoldPieceChanged {
    board: Entity,
    new_piece_kind: TetrominoKind,
}

#[derive(Component)]
pub struct Board {
    kind: TetrominoKind,
    pos: Vec2,
    rotation: TetrominoRotation,

    movement: Vec2,
    stationary_lock_delay: i32,
    lock_delay: i32,
    auto_shift_delay: i32,

    queue: VecDeque<TetrominoKind>,
    random_bag: ShuffleBag<TetrominoKind>,

    hold_piece: Option<TetrominoKind>,
    can_hold: bool,
}

impl Board {
    fn new<T: Rng>(mut rng: T) -> Self {
        let shuffle_bag = ShuffleBag::try_new(
            TetrominoKind::iter().collect::<Vec<TetrominoKind>>(),
            &mut rng,
        )
        .expect("Failed to create shuffle bag");
        let queue = VecDeque::from_iter(TetrominoKind::iter());

        Self {
            kind: TetrominoKind::I,
            pos: Default::default(),
            movement: Default::default(),
            rotation: Default::default(),

            stationary_lock_delay: Default::default(),
            lock_delay: Default::default(),
            auto_shift_delay: Default::default(),

            queue,
            random_bag: shuffle_bag,

            hold_piece: None,
            can_hold: true,
        }
    }
}

impl Board {
    pub fn spawn_next<T: Rng>(
        &mut self,
        commands: &mut Commands,
        self_entity: Entity,
        tilemap: &Tilemap,
        board_config: &BoardConfig,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
        tile_images: &Res<TileImages>,
        tile_outline_images: &Res<TileOutlineImages>,
        mut rng: T,
    ) {
        let Some(kind) = self.queue.pop_front() else {
            error_once!("Attempted to pop from empty piece queue!");
            return;
        };
        self.queue.push_back(*self.random_bag.pick(&mut rng));
        self.spawn(
            kind,
            commands,
            self_entity,
            tilemap,
            board_config,
            placed_tiles,
            tile_images,
            tile_outline_images,
        );
        self.can_hold = true;
    }

    pub fn spawn(
        &mut self,
        kind: TetrominoKind,
        commands: &mut Commands,
        self_entity: Entity,
        tilemap: &Tilemap,
        board_config: &BoardConfig,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
        tile_images: &Res<TileImages>,
        tile_outline_images: &Res<TileOutlineImages>,
    ) {
        self.kind = kind;
        self.pos = vec2(4.0, tilemap.size.y as f32 - 0.4);
        self.rotation = 0;
        self.lock_delay = board_config.lock_delay;

        if self.can_place(
            self_entity,
            tilemap,
            placed_tiles,
            self.get_snapped_pos(),
            self.rotation,
        ) {
            spawn_tetromino_tiles(
                commands,
                self,
                self_entity,
                tile_images,
                tile_outline_images,
            );
            spawn_ghost_tiles(commands, self, self_entity, tile_outline_images);
        } else {
            bevy::log::error_once!("Attempted to spawn tetromino at invalid position");
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

    pub fn place<T: QueryFilter, U: QueryFilter, R: Rng>(
        &mut self,
        commands: &mut Commands,
        self_entity: Entity,
        board_config: &BoardConfig,
        tilemap: &Tilemap,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
        tetromino_tiles: Query<(Entity, &Tile), T>,
        ghost_tiles: Query<(Entity, &Tile), U>,
        tile_images: &Res<TileImages>,
        tile_outline_images: &Res<TileOutlineImages>,
        rng: R,
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
                    Sprite::from_image(tile_images.0[&self.kind].clone()),
                ));
            }
        }
        clear_tetromino_tiles(commands, self_entity, tetromino_tiles);
        clear_ghost_tiles(commands, self_entity, ghost_tiles);
        self.spawn_next(
            commands,
            self_entity,
            tilemap,
            board_config,
            placed_tiles,
            tile_images,
            tile_outline_images,
            rng,
        );
    }

    fn get_hard_drop_pos(
        &self,
        self_entity: Entity,
        tilemap: &Tilemap,
        placed_tiles: Query<&Tile, With<PlacedTile>>,
    ) -> IVec2 {
        let mut result = self.get_snapped_pos();
        for y_pos in (0..self.get_snapped_pos().y).rev() {
            let new_pos = ivec2(self.get_snapped_pos().x, y_pos);
            if self.can_place(self_entity, tilemap, placed_tiles, new_pos, self.rotation) {
                result = new_pos;
            } else {
                break;
            }
        }
        result
    }

    fn get_snapped_pos(&self) -> IVec2 {
        snap_vec2(self.pos)
    }
}

fn snap_vec2(value: Vec2) -> IVec2 {
    value.round().as_ivec2()
}

// ========== Systems ==========

pub fn spawn_board<T: Rng>(
    commands: &mut Commands,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
    mut rng: T,
) {
    let board_backround_size = (size * tile_size).as_vec2();
    let scale = Vec3::splat(4.0);
    let tilemap = Tilemap { size, tile_size };
    let board_config = BoardConfig::default();
    let mut board = Board::new(&mut rng);

    let hold_size = uvec2(4, 4);
    let hold_background_size = (hold_size * tile_size).as_vec2();

    let entity = commands
        .spawn((
            Name::new("Board"),
            Mesh2d(meshes.add(Rectangle::from_size(board_backround_size))),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform::from_scale(scale),
            get_board_input_map(),
        ))
        .id();

    board.spawn_next(
        commands,
        entity,
        &tilemap,
        &board_config,
        placed_tiles,
        &tile_images,
        &tile_outline_images,
        rng,
    );

    commands
        .entity(entity)
        .insert((board, board_config, tilemap));
    commands.spawn((
        Tilemap {
            size: hold_size,
            tile_size,
        },
        HoldDisplay { board: entity },
        Mesh2d(meshes.add(Rectangle::from_size(hold_background_size))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-8.0 * 4.0 * 8.0, 8.0 * 4.0 * 8.0, 0.0).with_scale(scale),
    ));
}

fn move_lines_down(
    mut boards: Query<(Entity, &Tilemap), (With<Board>, Without<SkipUpdate>)>,
    mut tile_queries: ParamSet<(
        Query<(Entity, &Tile), With<PlacedTile>>,
        Query<(Entity, &mut Tile), With<PlacedTile>>,
    )>,
) {
    for (board_entity, tilemap) in boards.iter_mut() {
        let mut num_cleared_lines = 0;
        for y in 0..tilemap.size.y as i32 {
            let mut tiles_in_line: Vec<Entity> = vec![];

            for x in 0..tilemap.size.x as i32 {
                let tile_entities =
                    tilemap.get_tiles(board_entity, ivec2(x, y).as_vec2(), tile_queries.p0());
                for tile_entity in tile_entities {
                    tiles_in_line.push(tile_entity);
                }
            }

            if num_cleared_lines > 0 {
                for tile_entity in tiles_in_line.iter() {
                    if let Ok((_, mut tile)) = tile_queries.p1().get_mut(*tile_entity) {
                        tile.pos -= ivec2(0, num_cleared_lines).as_vec2();
                    }
                }
            }
            if tiles_in_line.is_empty() {
                num_cleared_lines += 1;
            }
        }
    }
}

fn apply_hold(
    mut commands: Commands,
    mut boards: Query<
        (
            Entity,
            &mut Board,
            &ActionState<Action>,
            &Tilemap,
            &BoardConfig,
        ),
        Without<SkipUpdate>,
    >,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
    mut random_source: ResMut<RandomSource>,
    mut hold_messages: MessageWriter<HoldPieceChanged>,
) {
    let rng = &mut random_source.0;

    for (board_entity, mut board, action_state, tilemap, board_config) in boards.iter_mut() {
        if action_state.just_pressed(&Action::Hold) && board.can_hold {
            board.can_hold = false;
            hold_messages.write(HoldPieceChanged {
                board: board_entity,
                new_piece_kind: board.kind,
            });

            let old_hold_piece = board.hold_piece;
            board.hold_piece = Some(board.kind);

            if let Some(old_hold_piece) = old_hold_piece {
                board.spawn(
                    old_hold_piece,
                    &mut commands,
                    board_entity,
                    tilemap,
                    board_config,
                    placed_tiles,
                    &tile_images,
                    &tile_outline_images,
                );
            } else {
                board.spawn_next(
                    &mut commands,
                    board_entity,
                    tilemap,
                    board_config,
                    placed_tiles,
                    &tile_images,
                    &tile_outline_images,
                    &mut *rng,
                );
                board.can_hold = false;
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

fn apply_auto_shift(
    mut boards: Query<(&ActionState<Action>, &mut Board, &BoardConfig), Without<SkipUpdate>>,
) {
    for (action_state, mut board, board_config) in boards.iter_mut() {
        let shift = if action_state.pressed(&Action::ShiftLeft) {
            -1
        } else if action_state.pressed(&Action::ShiftRight) {
            1
        } else {
            0
        };

        if shift == 0 {
            board.auto_shift_delay = board_config.auto_shift_delay;
            board.pos.x = board.get_snapped_pos().x as f32;
        } else if board.auto_shift_delay > 0 {
            board.auto_shift_delay -= 1;
        } else {
            board.movement.x += board_config.auto_shift_speed * shift as f32;
        }
    }
}

fn apply_soft_drop(
    mut boards: Query<(&ActionState<Action>, &mut Board, &BoardConfig), Without<SkipUpdate>>,
) {
    for (action_state, mut board, board_config) in boards.iter_mut() {
        if action_state.pressed(&Action::SoftDrop) {
            board.movement.y -= board_config.soft_drop_speed;
        }
    }
}

fn apply_hard_drop(
    mut commands: Commands,
    mut boards: Query<
        (
            Entity,
            &ActionState<Action>,
            &mut Board,
            &BoardConfig,
            &Tilemap,
        ),
        Without<SkipUpdate>,
    >,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    tetromino_tiles: Query<(Entity, &Tile), (With<TetrominoTile>, Without<PlacedTile>)>,
    ghost_tiles: Query<
        (Entity, &Tile),
        (With<GhostTile>, Without<TetrominoTile>, Without<PlacedTile>),
    >,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
    mut random_source: ResMut<RandomSource>,
) {
    let rng = &mut random_source.0;
    for (board_entity, action_state, mut board, board_config, tilemap) in boards.iter_mut() {
        if action_state.just_pressed(&Action::HardDrop) {
            board.pos = board
                .get_hard_drop_pos(board_entity, tilemap, placed_tiles)
                .as_vec2();
            board.place(
                &mut commands,
                board_entity,
                board_config,
                tilemap,
                placed_tiles,
                tetromino_tiles,
                ghost_tiles,
                &tile_images,
                &tile_outline_images,
                &mut *rng,
            );
        }
    }
}

pub fn apply_rotation(
    mut boards: Query<
        (
            Entity,
            &ActionState<Action>,
            &mut Board,
            &BoardConfig,
            &Tilemap,
        ),
        Without<SkipUpdate>,
    >,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
) {
    for (board_entity, action_state, mut board, board_config, tilemap) in boards.iter_mut() {
        let new_rotation = if action_state.just_pressed(&Action::RotateRight) {
            board.rotation + 1
        } else if action_state.just_pressed(&Action::RotateLeft) {
            board.rotation - 1
        } else {
            continue;
        };

        board.stationary_lock_delay = board_config.stationary_lock_delay;

        let offsets = get_tetromino_wall_kicks(board.rotation, new_rotation, board.kind);
        for offset in offsets.iter() {
            let new_pos = board.get_snapped_pos() + offset;
            if board.can_place(board_entity, tilemap, placed_tiles, new_pos, new_rotation) {
                board.pos += offset.as_vec2();
                board.rotation = new_rotation;
                return;
            }
        }
        bevy::log::warn_once!("All wall kicks failed!");
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
    mut boards: Query<
        (
            Entity,
            &mut Board,
            &BoardConfig,
            &Tilemap,
            &ActionState<Action>,
        ),
        Without<SkipUpdate>,
    >,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    tetromino_tiles: Query<(Entity, &Tile), (With<TetrominoTile>, Without<PlacedTile>)>,
    ghost_tiles: Query<
        (Entity, &Tile),
        (With<GhostTile>, Without<TetrominoTile>, Without<PlacedTile>),
    >,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
    mut random_source: ResMut<RandomSource>,
) {
    let rng = &mut random_source.0;

    for (board_entity, mut board, board_config, tilemap, action_state) in boards.iter_mut() {
        let pos_below = board.get_snapped_pos() - ivec2(0, 1);
        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            pos_below,
            board.rotation,
        ) || board.pos.y % 1.0 != 0.0
        {
            board.stationary_lock_delay = board_config.stationary_lock_delay;
            continue; // Piece can still move down
        }

        board.lock_delay -= 1;
        if action_state.pressed(&Action::ShiftLeft) || action_state.pressed(&Action::ShiftRight) {
            board.stationary_lock_delay = board_config.stationary_lock_delay;
        } else {
            board.stationary_lock_delay -= 1;
        }

        if board.lock_delay < 0 || board.stationary_lock_delay < 0 {
            board.place(
                &mut commands,
                board_entity,
                board_config,
                tilemap,
                placed_tiles,
                tetromino_tiles,
                ghost_tiles,
                &tile_images,
                &tile_outline_images,
                &mut *rng,
            );
        }
    }
}

fn remove_skip_update(mut commands: Commands, boards: Query<Entity, With<SkipUpdate>>) {
    for board_entity in boards {
        commands.entity(board_entity).remove::<SkipUpdate>();
    }
}
