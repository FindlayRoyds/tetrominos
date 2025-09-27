use bevy::prelude::*;

use crate::{board::SkipUpdate, tiles::Tile};

#[derive(Component)]
pub struct LineClearTile {
    pub fade_time: i32, // Below what lifetime the tile should start to fade
    pub lifetime: i32,
}

pub fn apply_line_clear_skip_update(
    mut commands: Commands,
    line_clear_tiles: Query<&Tile, With<LineClearTile>>,
) {
    for tile in line_clear_tiles {
        commands.entity(tile.tilemap).insert(SkipUpdate);
    }
}

pub fn apply_line_clear_lifetime(
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

pub fn apply_line_clear_visuals(mut line_clear_tiles: Query<(&mut Transform, &LineClearTile)>) {
    for (mut transform, line_clear_tile) in line_clear_tiles.iter_mut() {
        transform.scale = Vec3::splat(
            (line_clear_tile.lifetime as f32 / line_clear_tile.fade_time as f32).min(1.0),
        );
    }
}
