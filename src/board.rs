use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_shuffle_bag::ShuffleBag;
use leafwing_input_manager::prelude::ActionState;
use rand::{Rng, seq::SliceRandom};
use strum::IntoEnumIterator;

mod board_config;
mod ghost_tile;
pub mod hold_display;
mod line_clear;
mod outline;
pub mod placed_tile;
pub mod queue_display;
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
        queue_display::{QueueDisplay, QueueDisplayPlugin},
        tetromino_data::{
            TetrominoKind, TetrominoRotation, get_tetromino_shape, get_tetromino_start_piece,
            get_tetromino_wall_kicks,
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
            QueueDisplayPlugin,
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
                    place_tetrominos,
                    spawn_next_tetrominos,
                    spawn_tetrominos,
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
        .add_message::<HoldPieceChanged>()
        .add_message::<TetrominoQueueChanged>()
        .add_message::<PlaceTetromino>()
        .add_message::<SpawnNextTetromino>()
        .add_message::<SpawnTetromino>();
    }
}

pub type TetrominoQueue = VecDeque<TetrominoKind>;

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

#[derive(Message)]
pub struct TetrominoQueueChanged {
    board: Entity,
    new_queue: TetrominoQueue,
}

#[derive(Message)]
pub struct PlaceTetromino {
    board: Entity,
}

#[derive(Message)]
pub struct SpawnNextTetromino {
    board: Entity,
}

#[derive(Message)]
pub struct SpawnTetromino {
    board: Entity,
    kind: TetrominoKind,
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

    queue: TetrominoQueue,
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
        let mut kinds: Vec<TetrominoKind> = TetrominoKind::iter().collect();
        kinds.shuffle(&mut rng);
        let mut queue = TetrominoQueue::from(kinds);
        queue[0] = get_tetromino_start_piece(&mut rng);

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
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    mut rng: T,
    mut spawn_next_messages: MessageWriter<SpawnNextTetromino>,
) {
    let board_backround_size = (size * tile_size).as_vec2();
    let scale = Vec3::splat(4.0);
    let tilemap = Tilemap { size, tile_size };
    let board_config = BoardConfig::default();
    let board = Board::new(&mut rng);

    let hold_display_size = uvec2(4, 4);
    let hold_background_size = (hold_display_size * tile_size).as_vec2();

    let queue_display_length = board_config.queue_display_length;
    let queue_display_size = uvec2(4, queue_display_length * 4);
    let queue_background_size = (queue_display_size * tile_size).as_vec2();

    let entity = commands
        .spawn((
            Name::new("Board"),
            Mesh2d(meshes.add(Rectangle::from_size(board_backround_size))),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform::from_scale(scale),
            get_board_input_map(),
        ))
        .id();

    spawn_next_messages.write(SpawnNextTetromino { board: entity });

    commands
        .entity(entity)
        .insert((board, board_config, tilemap));

    // Hold display
    commands.spawn((
        Tilemap {
            size: hold_display_size,
            tile_size,
        },
        HoldDisplay { board: entity },
        Mesh2d(meshes.add(Rectangle::from_size(hold_background_size))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-8.0 * 4.0 * 8.0, 8.0 * 4.0 * 8.0, 0.0).with_scale(scale),
    ));

    // Queue Display
    commands.spawn((
        Tilemap {
            size: queue_display_size,
            tile_size,
        },
        QueueDisplay {
            board: entity,
            length: queue_display_length,
        },
        Mesh2d(meshes.add(Rectangle::from_size(queue_background_size))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(8.0 * 4.0 * 8.0, 2.0 * 4.0 * 8.0, 0.0).with_scale(scale),
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
    mut boards: Query<(Entity, &mut Board, &ActionState<Action>), Without<SkipUpdate>>,
    mut hold_messages: MessageWriter<HoldPieceChanged>,
    mut spawn_next_messages: MessageWriter<SpawnNextTetromino>,
    mut spawn_messages: MessageWriter<SpawnTetromino>,
) {
    for (board_entity, mut board, action_state) in boards.iter_mut() {
        if action_state.just_pressed(&Action::Hold) && board.can_hold {
            board.can_hold = false;
            hold_messages.write(HoldPieceChanged {
                board: board_entity,
                new_piece_kind: board.kind,
            });

            let old_hold_piece = board.hold_piece;
            board.hold_piece = Some(board.kind);

            if let Some(old_hold_piece) = old_hold_piece {
                spawn_messages.write(SpawnTetromino {
                    board: board_entity,
                    kind: old_hold_piece,
                });
            } else {
                spawn_next_messages.write(SpawnNextTetromino {
                    board: board_entity,
                });
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
    mut boards: Query<(Entity, &ActionState<Action>, &mut Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    mut place_messages: MessageWriter<PlaceTetromino>,
) {
    for (board_entity, action_state, mut board, tilemap) in boards.iter_mut() {
        if action_state.just_pressed(&Action::HardDrop) {
            board.pos = board
                .get_hard_drop_pos(board_entity, tilemap, placed_tiles)
                .as_vec2();
            place_messages.write(PlaceTetromino {
                board: board_entity,
            });
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
    mut place_messages: MessageWriter<PlaceTetromino>,
) {
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
            place_messages.write(PlaceTetromino {
                board: board_entity,
            });
        }
    }
}

fn place_tetrominos(
    boards: Query<(Entity, &Board, &Tilemap), Without<SkipUpdate>>,
    mut place_messages: MessageReader<PlaceTetromino>,
    mut commands: Commands,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    tetromino_tiles: Query<(Entity, &Tile), (With<TetrominoTile>, Without<PlacedTile>)>,
    ghost_tiles: Query<
        (Entity, &Tile),
        (With<GhostTile>, Without<TetrominoTile>, Without<PlacedTile>),
    >,
    tile_images: Res<TileImages>,
    mut spawn_next_messages: MessageWriter<SpawnNextTetromino>,
) {
    for message in place_messages.read() {
        let Ok((board_entity, board, tilemap)) = boards.get(message.board) else {
            bevy::log::error_once!("Failed to get board when spawning next tetromino!");
            break;
        };
        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            board.get_snapped_pos(),
            board.rotation,
        ) {
            for offset in get_tetromino_shape(board.kind, board.rotation) {
                let pos = board.get_snapped_pos() + offset;
                commands.spawn((
                    Name::new("PlacedTile"),
                    Tile {
                        pos: pos.as_vec2(),
                        tilemap: board_entity,
                    },
                    PlacedTile,
                    ChildOf(board_entity),
                    Sprite::from_image(tile_images.0[&board.kind].clone()),
                ));
            }
        }
        clear_tetromino_tiles(&mut commands, board_entity, tetromino_tiles);
        clear_ghost_tiles(&mut commands, board_entity, ghost_tiles);
        spawn_next_messages.write(SpawnNextTetromino {
            board: board_entity,
        });
    }
}

fn spawn_next_tetrominos(
    mut boards: Query<(Entity, &mut Board), Without<SkipUpdate>>,
    mut spawn_next_messages: MessageReader<SpawnNextTetromino>,
    mut spawn_messages: MessageWriter<SpawnTetromino>,
    mut queue_messages: MessageWriter<TetrominoQueueChanged>,
    mut random_source: ResMut<RandomSource>,
) {
    let mut rng = &mut random_source.0;

    for message in spawn_next_messages.read() {
        let Ok((board_entity, mut board)) = boards.get_mut(message.board) else {
            bevy::log::error_once!("Failed to get board when spawning next tetromino!");
            break;
        };

        let Some(kind) = board.queue.pop_front() else {
            error_once!("Attempted to pop from empty piece queue!");
            return;
        };
        let picked_tetromino = *board.random_bag.pick(&mut rng);
        board.queue.push_back(picked_tetromino);
        spawn_messages.write(SpawnTetromino {
            board: board_entity,
            kind,
        });
        board.can_hold = true;
        queue_messages.write(TetrominoQueueChanged {
            board: board_entity,
            new_queue: board.queue.clone(),
        });
    }
}

fn spawn_tetrominos(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &Tilemap, &BoardConfig), Without<SkipUpdate>>,
    mut messages: MessageReader<SpawnTetromino>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
    tile_images: Res<TileImages>,
    tile_outline_images: Res<TileOutlineImages>,
) {
    for message in messages.read() {
        let Ok((board_entity, mut board, tilemap, board_config)) = boards.get_mut(message.board)
        else {
            bevy::log::error_once!("Failed to get board when spawning next tetromino!");
            break;
        };

        board.kind = message.kind;
        board.pos = vec2(4.0, tilemap.size.y as f32 - 0.4);
        board.rotation = 0;
        board.lock_delay = board_config.lock_delay;

        if board.can_place(
            board_entity,
            tilemap,
            placed_tiles,
            board.get_snapped_pos(),
            board.rotation,
        ) {
            spawn_tetromino_tiles(
                &mut commands,
                &board,
                board_entity,
                &tile_images,
                &tile_outline_images,
            );
            spawn_ghost_tiles(&mut commands, &board, board_entity, &tile_outline_images);
        } else {
            bevy::log::error_once!("Attempted to spawn tetromino at invalid position");
        }
    }
}

fn remove_skip_update(mut commands: Commands, boards: Query<Entity, With<SkipUpdate>>) {
    for board_entity in boards {
        commands.entity(board_entity).remove::<SkipUpdate>();
    }
}
