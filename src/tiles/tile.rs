use bevy::prelude::*;

use crate::{tiles::Tilemap, try_unwrap};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_tile_transforms, update_tile_visibility).in_set(TileVisuals),
        );
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Tile {
    pub pos: IVec2,
    pub offset: Vec2,
    pub tilemap: Entity,
}

#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileVisuals;

fn update_tile_transforms(mut query: Query<(&Tile, &mut Transform)>, tilemaps: Query<&Tilemap>) {
    for (tile, mut transform) in query.iter_mut() {
        let tilemap = try_unwrap!(tilemaps.get(tile.tilemap), "No tilemap in tile_transforms");
        transform.translation =
            ((tile.pos.as_vec2() - tilemap.size.as_vec2() / 2.0 + vec2(0.5, 0.5) + tile.offset)
                * tilemap.tile_size.as_vec2())
            .extend(0.0);
    }
}

fn update_tile_visibility(mut tiles: Query<(&Tile, &mut Visibility)>, tilemaps: Query<&Tilemap>) {
    for (tile, mut visibility) in tiles.iter_mut() {
        let tilemap = try_unwrap!(tilemaps.get(tile.tilemap), "No tilemap in tile_visibility");
        *visibility = if tilemap.is_in_bounds(tile.pos) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
