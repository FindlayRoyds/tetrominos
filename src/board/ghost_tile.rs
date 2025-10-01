use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::{
    board::{
        Board, SkipUpdate,
        placed_tile::PlacedTile,
        tetromino_data::{get_tetromino_color, get_tetromino_shape},
    },
    tiles::{Tile, Tilemap},
};

pub struct GhostTilePlugin;

impl Plugin for GhostTilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_ghost_tile_positions.in_set(GhostTileVisuals),
        );
    }
}

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostTileVisuals;

#[derive(Component)]
pub struct GhostTile {
    offset_index: usize,
}

pub fn spawn_ghost_tiles(
    commands: &mut Commands,
    board: &Board,
    board_entity: Entity,
    asset_server: &Res<AssetServer>,
) {
    for (index, offset) in get_tetromino_shape(board.kind, board.rotation)
        .iter()
        .enumerate()
    {
        let pos = (board.get_snapped_pos() + offset).as_vec2();
        let color_str = get_tetromino_color(board.kind);

        commands.spawn((
            Name::new("GhostTile"),
            Tile {
                pos,
                tilemap: board_entity,
            },
            GhostTile {
                offset_index: index,
            },
            ChildOf(board_entity),
            Sprite {
                image: asset_server.load(format!("tiles/outline_{}.png", color_str)),
                // color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                ..Default::default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        ));
    }
}

pub fn clear_ghost_tiles<T: QueryFilter>(
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

fn update_ghost_tile_positions(
    mut ghost_tiles: Query<(&mut Tile, &GhostTile), Without<PlacedTile>>,
    boards: Query<(&Board, &Tilemap), Without<SkipUpdate>>,
    placed_tiles: Query<&Tile, With<PlacedTile>>,
) {
    for (mut tile, ghost_tile) in ghost_tiles.iter_mut() {
        let Ok((board, tilemap)) = boards.get(tile.tilemap) else {
            continue; // Board likely has SkipUpdate component
        };

        let offsets = get_tetromino_shape(board.kind, board.rotation);
        tile.pos = (board.get_hard_drop_pos(tile.tilemap, tilemap, placed_tiles)
            + offsets[ghost_tile.offset_index])
            .as_vec2();
    }
}
