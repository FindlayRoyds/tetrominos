use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{
    board::{
        Board, BoardUpdateSystems,
        board_config::BoardConfig,
        outline::TetrominoTileOutline,
        tetromino_data::get_tetromino_shape,
        tile_assets::{TileImages, TileOutlineImages},
    },
    tiles::Tile,
};

pub struct TetrominoTilePlugin;

impl Plugin for TetrominoTilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update_tetromino_tile_positions, apply_lock_delay_visuals)
                .chain()
                .after(BoardUpdateSystems),
        );
    }
}

#[derive(Component)]
pub struct TetrominoTile {
    pub offset_index: usize,
}

pub fn spawn_tetromino_tiles(
    commands: &mut Commands,
    board: &Board,
    board_entity: Entity,
    tile_images: &Res<TileImages>,
    tile_outline_images: &Res<TileOutlineImages>,
) {
    for (index, offset) in get_tetromino_shape(board.kind, board.rotation)
        .iter()
        .enumerate()
    {
        let pos = (board.get_snapped_pos() + offset).as_vec2();
        commands.spawn((
            Name::new("TetrominoTile"),
            Tile {
                pos,
                tilemap: board_entity,
            },
            TetrominoTile {
                offset_index: index,
            },
            ChildOf(board_entity),
            Sprite::from_image(tile_images.0[&board.kind].clone()),
        ));
        commands.spawn((
            Name::new("TetrominoTileOutline"),
            Tile {
                pos,
                tilemap: board_entity,
            },
            TetrominoTile {
                offset_index: index,
            },
            TetrominoTileOutline,
            ChildOf(board_entity),
            Sprite::from_image(tile_outline_images.0[&board.kind].clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        ));
    }
}

pub fn clear_tetromino_tiles<T: QueryFilter>(
    commands: &mut Commands,
    board_entity: Entity,
    tiles: Query<(Entity, &Tile), T>,
) {
    for (tile_entity, tile) in tiles {
        if tile.tilemap != board_entity {
            continue;
        }

        commands.entity(tile_entity).despawn()
    }
}

fn update_tetromino_tile_positions(
    mut tiles: Query<(&mut Tile, &TetrominoTile)>,
    boards: Query<&Board>,
) {
    for (mut tile, tetromino_tile) in tiles.iter_mut() {
        let Ok(board) = boards.get(tile.tilemap) else {
            bevy::log::warn_once!("Failed to get board in update tetromino tile positions");
            continue;
        };

        let offsets = get_tetromino_shape(board.kind, board.rotation);
        tile.pos = (board.get_snapped_pos() + offsets[tetromino_tile.offset_index]).as_vec2();
    }
}

fn apply_lock_delay_visuals(
    mut tiles: Query<(&Tile, &mut Sprite), (With<TetrominoTile>, Without<TetrominoTileOutline>)>,
    boards: Query<(&Board, &BoardConfig)>,
) {
    for (tile, mut sprite) in tiles.iter_mut() {
        let Ok((board, board_config)) = boards.get(tile.tilemap) else {
            bevy::log::error_once!("Failed to get board in apply_lock_delay_visuals");
            continue;
        };

        let normal_effect =
            board.stationary_lock_delay as f32 / board_config.stationary_lock_delay as f32;
        let stationary_effect = board.lock_delay as f32 / board_config.lock_delay as f32;

        sprite.color.set_alpha(normal_effect.min(stationary_effect));
    }
}
