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
    pub tilemap: Entity,
}

// Todo return entity commands, don't include placed in arguments
pub fn spawn_tile(
    commands: &mut Commands,
    pos: IVec2,
    tilemap: Entity,
    placed: bool,
    asset_server: &Res<AssetServer>,
) -> Entity {
    let mut entity_commands = commands.spawn((
        Name::new("Tile"),
        Tile { pos, tilemap },
        ChildOf(tilemap),
        Sprite::from_image(asset_server.load("tiles/tile.png")),
    ));

    if placed {
        entity_commands.insert(crate::board::placed_tile::PlacedTile);
    }

    entity_commands.id()
}

#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileVisuals;

fn update_tile_transforms(mut query: Query<(&Tile, &mut Transform)>, tilemaps: Query<&Tilemap>) {
    for (tile, mut transform) in query.iter_mut() {
        let tilemap = try_unwrap!(tilemaps.get(tile.tilemap), "No tilemap in tile_transforms");
        transform.translation = ((tile.pos.as_vec2() - tilemap.size.as_vec2() / 2.0
            + vec2(0.5, 0.5))
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
