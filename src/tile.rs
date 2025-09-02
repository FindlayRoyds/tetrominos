use bevy::prelude::*;

use crate::board::Board;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_tile_transforms, update_tile_visibility).in_set(TileVisuals),
        )
        .add_observer(on_add_placed_tile)
        .add_observer(on_remove_tile);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct Tile {
    pos: IVec2,
    pub board_entity: Entity,
}

impl Tile {
    #[allow(dead_code)]
    pub fn get_pos(&self) -> IVec2 {
        self.pos
    }

    #[allow(dead_code)]
    pub fn set_pos(&mut self, pos: IVec2, boards_query: &mut Query<&mut Board>) {
        let mut board = boards_query
            .get_mut(self.board_entity)
            .expect("Board not found");
        assert!(board.get_tile(pos).is_some(), "Tile pos already occupied");

        let entity = board.remove_tile(pos).expect("Tile not found");
        board.set_tile(pos, entity);
        self.pos = pos;
    }
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
        Sprite::from_image(tile_image),
    ));

    if placed {
        tile_commands.insert(PlacedTile);
    }

    return tile_commands.id();
}

/// Systems that only read tile components, run after updates to tiles
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileVisuals;

/// Systems that make changes to tile components
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileUpdates;

fn update_tile_transforms(mut query: Query<(&Tile, &mut Transform)>, boards: Query<&Board>) {
    for (tile, mut transform) in query.iter_mut() {
        let board = boards.get(tile.board_entity).expect("Failed to get board");
        transform.translation =
            ((tile.pos.as_vec2() - board.size.as_vec2() / 2.0 + vec2(0.5, 0.5)) * 8.0).extend(0.0);
    }
}

fn update_tile_visibility(mut tiles: Query<(&Tile, &mut Visibility)>, boards: Query<&Board>) {
    for (tile, mut visibility) in tiles.iter_mut() {
        let board = boards
            .get(tile.board_entity)
            .expect("Failed to get board for tile");
        *visibility = if board.is_in_bounds(tile.pos) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn on_add_placed_tile(
    trigger: Trigger<OnAdd, PlacedTile>,
    tiles: Query<&Tile>,
    mut boards: Query<&mut Board>,
) {
    let tile = tiles.get(trigger.target()).expect("Failed to get tile");
    let mut board = boards
        .get_mut(tile.board_entity)
        .expect("Failed to get board");
    assert!(
        board.get_tile(tile.pos).is_none(),
        "Tile pos already occupied"
    );
    board.set_tile(tile.pos, trigger.target());
}

fn on_remove_tile(
    trigger: Trigger<OnRemove, PlacedTile>,
    tiles: Query<&Tile>,
    mut boards: Query<&mut Board>,
) {
    let tile = tiles.get(trigger.target()).expect("Failed to get tile");
    let mut board = boards
        .get_mut(tile.board_entity)
        .expect("Failed to get board");
    assert!(
        board
            .get_tile(tile.pos)
            .expect("Failed to get tile from board")
            == trigger.target(),
        "Incorrect tile entity in board"
    );
    board.remove_tile(tile.pos);
}
