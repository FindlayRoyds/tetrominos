use bevy::{platform::collections::HashMap, prelude::*};

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
pub struct Board {
    pub size: UVec2,
    tiles: HashMap<IVec2, Entity>,
}

impl Board {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            tiles: HashMap::new(),
        }
    }

    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        self.tiles.get(&pos).copied()
    }

    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
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
        assert!(!board.tiles.contains_key(&pos), "Tile pos already occupied");

        let entity = board.tiles.remove(&self.pos).expect("Tile not found");
        board.tiles.insert(pos, entity);
        self.pos = pos;
    }
}

// Should be updated to return entity commands
pub fn spawn_tile(
    commands: &mut Commands,
    pos: IVec2,
    board_entity: Entity,
    asset_server: &Res<AssetServer>,
) -> Entity {
    let tile_image = asset_server.load("tiles/tile.png");

    let tile_entity = commands
        .spawn((
            Name::new("Tile"),
            Tile { pos, board_entity },
            Sprite::from_image(tile_image),
        ))
        .id();

    return tile_entity;
}

/// Systems that only read tile components, run after updates to tiles
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileVisuals;

/// Systems that make changes to tile components
#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TileUpdates;

fn update_tile_transforms(mut query: Query<(&Tile, &mut Transform)>) {
    for (tile, mut transform) in query.iter_mut() {
        transform.translation = (tile.pos.as_vec2() * 8.0).extend(0.0);
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

fn on_add_tile(trigger: Trigger<OnAdd, Tile>, tiles: Query<&Tile>, mut boards: Query<&mut Board>) {
    let tile = tiles.get(trigger.target()).expect("Failed to get tile");
    let mut board = boards
        .get_mut(tile.board_entity)
        .expect("Failed to get board");
    assert!(
        board.get_tile(tile.pos).is_none(),
        "Tile pos already occupied"
    );
    board.tiles.insert(tile.pos, trigger.target());
}

fn on_remove_tile(
    trigger: Trigger<OnRemove, Tile>,
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
    board.tiles.remove(&tile.pos);
}
