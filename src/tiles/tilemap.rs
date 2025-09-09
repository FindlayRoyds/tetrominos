use bevy::prelude::*;

use crate::tiles::Tile;

#[derive(Component)]
#[require(Transform)]
pub struct Tilemap {
    pub size: UVec2,
    pub tile_size: UVec2,
}

impl Tilemap {
    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }

    pub fn get_tile<F>(
        &self,
        self_entity: Entity,
        pos: IVec2,
        tiles: Query<(Entity, &Tile), F>,
    ) -> Option<Entity>
    where
        F: bevy::ecs::query::QueryFilter,
    {
        tiles
            .iter()
            .find(|(_, tile)| tile.tilemap == self_entity && tile.pos == pos)
            .map(|(entity, _)| entity)
    }
}
