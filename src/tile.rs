use bevy::prelude::*;

use crate::{board::Board, try_unwrap};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_tile_transforms, update_tile_visibility).in_set(TileVisuals),
        )
        .add_observer(on_add_tile)
        .add_observer(on_remove_tile);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Tile {
    pos: IVec2,
    pub board_entity: Entity,
}

#[derive(Component)]
pub struct PlacedTile;

// Should be updated to return entity commands
pub fn spawn_tile(
    commands: &mut Commands,
    pos: IVec2,
    board_entity: Entity,
    placed: bool,
    asset_server: &Res<AssetServer>,
) -> Entity {
    let tile_image = asset_server.load("tiles/tile.png");

    let mut tile_commands = commands.spawn((
        Name::new("Tile"),
        Tile { pos, board_entity },
        ChildOf(board_entity),
        Sprite::from_image(tile_image),
    ));
    // tile_commands.insert();

    if placed {
        tile_commands.insert(PlacedTile);
    }

    tile_commands.id()
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

fn on_add_tile(
    trigger: Trigger<OnAdd, PlacedTile>,
    tiles: Query<&Tile>,
    mut boards: Query<&mut Board>,
) {
    let tile = try_unwrap!(tiles.get(trigger.target()), "No tile in on_add_tile");
    let mut board = try_unwrap!(boards.get_mut(tile.board_entity), "No board in on_add_tile");
    if board.get_tile(tile.pos).is_some() {
        bevy::log::error!("Tile pos already occupied in on_add_placed_tile")
    }
    board.set_tile(tile.pos, trigger.target());
}

fn on_remove_tile(
    trigger: Trigger<OnRemove, PlacedTile>,
    tiles: Query<&Tile>,
    mut boards: Query<&mut Board>,
) {
    let tile = try_unwrap!(tiles.get(trigger.target()), "No tile in on_remove_tile");
    let mut board = try_unwrap!(
        boards.get_mut(tile.board_entity),
        "Failed to get board in on_remove_tile"
    );
    if try_unwrap!(board.get_tile(tile.pos), "No board in on_remove_tile") != trigger.target() {
        bevy::log::error!("Incorrect tile entity in board in on_remove_tile");
    }

    board.remove_tile(tile.pos);
}
