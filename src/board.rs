use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Component)]
pub struct Board {
    pub size: UVec2,
    pub tile_size: UVec2,

    tiles: HashMap<IVec2, Entity>,
}

impl Board {
    pub fn new(size: UVec2, tile_size: UVec2) -> Self {
        Self {
            size,
            tiles: HashMap::new(),
            tile_size,
        }
    }

    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        self.tiles.get(&pos).copied()
    }

    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }

    pub fn set_tile(&mut self, pos: IVec2, entity: Entity) {
        assert!(self.is_in_bounds(pos), "Tile position out of bounds");
        assert!(
            !self.tiles.contains_key(&pos),
            "Tile position already occupied"
        );
        self.tiles.insert(pos, entity);
    }

    pub fn remove_tile(&mut self, pos: IVec2) -> Option<Entity> {
        return self.tiles.remove(&pos);
    }
}

pub fn spawn_board(
    commands: &mut Commands,
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("Board"),
        Board::new(size, tile_size),
        Mesh2d(meshes.add(Rectangle::new(
            (size.x * tile_size.x) as f32,
            (size.y * tile_size.x) as f32,
        ))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_scale(vec3(4.0, 4.0, 4.0)),
    ));
}
