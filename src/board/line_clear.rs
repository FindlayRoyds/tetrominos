use bevy::prelude::*;

use crate::{
    board::{Board, SkipUpdate, board_config::BoardConfig, placed_tile::PlacedTile},
    tiles::{Tile, Tilemap},
};

pub struct LineClearPlugin;

impl Plugin for LineClearPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            FixedUpdate,
            (
                apply_line_clear_lifetime,
                apply_line_clear_skip_update,
                apply_line_clear_visuals,
            )
                .in_set(LineClearVisuals),
        );
    }
}

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineClearVisuals;

#[derive(Resource)]
pub struct LineClearSprite(Handle<Image>);

#[derive(Component)]
pub struct LineClearTile {
    pub fade_time: i32, // Below what lifetime the tile should start to fade
    pub lifetime: i32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let line_clear_sprite: Handle<Image> = asset_server.load("tiles/line_clear.png");
    commands.insert_resource(LineClearSprite(line_clear_sprite));
}

fn apply_line_clear_skip_update(
    mut commands: Commands,
    line_clear_tiles: Query<&Tile, With<LineClearTile>>,
) {
    for tile in line_clear_tiles {
        commands.entity(tile.tilemap).insert(SkipUpdate);
    }
}

fn apply_line_clear_lifetime(
    mut commands: Commands,
    mut line_clear_tiles: Query<(Entity, &mut LineClearTile)>,
) {
    for (tile_entity, mut line_clear_tile) in line_clear_tiles.iter_mut() {
        line_clear_tile.lifetime -= 1;
        if line_clear_tile.lifetime < 0 {
            commands.entity(tile_entity).despawn();
        }
    }
}

fn apply_line_clear_visuals(mut line_clear_tiles: Query<(&mut Transform, &LineClearTile)>) {
    for (mut transform, line_clear_tile) in line_clear_tiles.iter_mut() {
        transform.scale = Vec3::splat(
            (line_clear_tile.lifetime as f32 / line_clear_tile.fade_time as f32).min(1.0),
        );
    }
}

/// In BoardUpdates set
pub fn clear_lines(
    mut commands: Commands,
    mut boards: Query<(Entity, &Tilemap, &BoardConfig), (With<Board>, Without<SkipUpdate>)>,
    placed_tiles: Query<(Entity, &Tile), With<PlacedTile>>,
    line_clear_sprite: Res<LineClearSprite>,
) {
    for (board_entity, tilemap, board_config) in boards.iter_mut() {
        for y in 0..tilemap.size.y as i32 {
            let mut clear_line = true;
            let mut tiles_to_clear: Vec<Entity> = vec![];

            for x in 0..tilemap.size.x as i32 {
                let tile_entities =
                    tilemap.get_tiles(board_entity, ivec2(x, y).as_vec2(), placed_tiles);
                if !tile_entities.is_empty() {
                    for tile_entity in tile_entities {
                        tiles_to_clear.push(tile_entity);
                    }
                } else {
                    clear_line = false;
                }
            }

            if clear_line {
                for tile_entity in tiles_to_clear {
                    commands.entity(tile_entity).despawn();
                }

                for x in 0..tilemap.size.x as i32 {
                    commands.spawn((
                        Name::new("LineClearTile"),
                        Tile {
                            pos: ivec2(x, y).as_vec2(),
                            tilemap: board_entity,
                        },
                        LineClearTile {
                            fade_time: board_config.line_clear_fade_time,
                            lifetime: board_config.line_clear_delay
                                + board_config.line_clear_horizontal_delay * x,
                        },
                        ChildOf(board_entity),
                        Sprite::from_image(line_clear_sprite.0.clone()),
                    ));
                }
            }
        }
    }
}
