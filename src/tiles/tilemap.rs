use bevy::{ecs::query::QueryFilter, prelude::*};

use crate::tiles::Tile;

#[derive(Component)]
#[require(Transform)]
pub struct Tilemap {
    pub size: UVec2,
    pub tile_size: UVec2,
}

impl Tilemap {
    #[allow(dead_code)]
    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }

    /// Be careful of float comparisons. Shouldn't be an issue on integer position values.
    #[allow(dead_code)]
    pub fn get_tiles<F: QueryFilter>(
        &self,
        self_entity: Entity,
        pos: Vec2,
        tiles: Query<(Entity, &Tile), F>,
    ) -> Vec<Entity> {
        tiles
            .iter()
            .filter_map(|(entity, tile)| {
                if tile.tilemap == self_entity && tile.pos == pos {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Be careful of float comparisons. Shouldn't be an issue on integer position values.
    #[allow(dead_code)]
    pub fn is_tile<F: QueryFilter>(
        &self,
        self_entity: Entity,
        pos: Vec2,
        tiles: Query<&Tile, F>,
    ) -> bool {
        tiles
            .iter()
            .any(|tile| tile.tilemap == self_entity && tile.pos == pos)
    }
}
