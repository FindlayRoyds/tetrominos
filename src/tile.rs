use bevy::prelude::*;

use crate::{board::Board, try_unwrap};

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
    pub board_entity: Entity,
    pub placed: bool,
}

pub fn spawn_tile(
    commands: &mut Commands,
    pos: IVec2,
    board_entity: Entity,
    placed: bool,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn((
            Name::new("Tile"),
            Tile {
                pos,
                board_entity,
                placed,
            },
            ChildOf(board_entity),
            Sprite::from_image(asset_server.load("tiles/tile.png")),
        ))
        .id()
}

/// Systems that only read tile components, run after updates to tiles
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileVisuals;

/// Systems that make changes to tile components
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileUpdates;

fn update_tile_transforms(mut query: Query<(&Tile, &mut Transform)>, boards: Query<&Board>) {
    for (tile, mut transform) in query.iter_mut() {
        let board = try_unwrap!(boards.get(tile.board_entity), "No board in tile_transforms");
        transform.translation = ((tile.pos.as_vec2() - board.size.as_vec2() / 2.0
            + vec2(0.5, 0.5))
            * board.tile_size.as_vec2())
        .extend(0.0);
    }
}

fn update_tile_visibility(mut tiles: Query<(&Tile, &mut Visibility)>, boards: Query<&Board>) {
    for (tile, mut visibility) in tiles.iter_mut() {
        let board = try_unwrap!(boards.get(tile.board_entity), "No board in tile_visibility");
        *visibility = if board.is_in_bounds(tile.pos) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
