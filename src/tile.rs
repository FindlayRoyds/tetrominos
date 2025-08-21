use bevy::{platform::collections::HashMap, prelude::*};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_tile_visual_positions);
    }
}

// ====== components and other stuff ======

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
    pub board: Entity,
}

impl Tile {
    pub fn get_pos(&self) -> IVec2 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: IVec2, boards_query: &mut Query<&mut Board>) {
        let mut board = boards_query.get_mut(self.board).expect("Board not found");
        assert!(!board.tiles.contains_key(&pos), "Tile pos already occupied");

        let entity = board.tiles.remove(&self.pos).expect("Tile not found");
        board.tiles.insert(pos, entity);
        self.pos = pos;
    }
}

pub struct SpawnTile {
    pub pos: IVec2,
    pub board: Entity,
}

impl Command for SpawnTile {
    fn apply(self, world: &mut World) -> () {
        let asset_server = world.resource::<AssetServer>();
        let tile_image = asset_server.load("tiles/tile.png");

        {
            let board_entity = world.entity(self.board);
            let board = board_entity.get::<Board>().expect("Board not found");
            assert!(
                board.get_tile(self.pos).is_none(),
                "Tile pos already occupied"
            );
            assert!(board.is_in_bounds(self.pos), "Tile pos is out of bounds");
        }

        let tile_entity = world
            .spawn((
                Name::new("Tile"),
                Tile {
                    pos: self.pos,
                    board: self.board,
                },
                Sprite::from_image(tile_image),
            ))
            .id();

        let mut board_entity = world.entity_mut(self.board);
        let mut board = board_entity.get_mut::<Board>().expect("Board not found");
        board.tiles.insert(self.pos, tile_entity);
    }
}

// ====== systems ======

fn update_tile_visual_positions(mut query: Query<(&Tile, &mut Transform)>) {
    for (tile, mut transform) in query.iter_mut() {
        transform.translation = (tile.pos.as_vec2() * 8.0).extend(0.0);
    }
}
