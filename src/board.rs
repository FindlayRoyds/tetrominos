use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    tetrominoes::{Tetromino, TetrominoKind, TetrominoRotation, get_tetromino_shape},
    tile::spawn_tile,
    try_unwrap,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_next_tetromino.in_set(TetrominoSpawning));
    }
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TetrominoSpawning;

#[derive(Component)]
pub struct Board {
    pub size: UVec2,
    pub tile_size: UVec2,

    // Inputs
    pub shift: i32,
    pub auto_shift: i32,
    pub auto_shift_delay: i32,
    pub soft_drop: bool,
    pub hard_drop: bool,
    pub rotate: i32,

    tiles: HashMap<IVec2, Entity>,
}

impl Board {
    pub fn new(size: UVec2, tile_size: UVec2) -> Self {
        Self {
            size,
            tile_size,

            shift: 0,
            auto_shift_delay: 0, // TODO set to config value, fine for now though
            auto_shift: 0,
            soft_drop: false,
            hard_drop: false,
            rotate: 0,

            tiles: HashMap::new(),
        }
    }

    pub fn get_tile(&self, pos: IVec2) -> Option<Entity> {
        self.tiles.get(&pos).copied()
    }

    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }

    pub fn set_tile(&mut self, pos: IVec2, entity: Entity) {
        if !self.is_in_bounds(pos) {
            bevy::log::error!("Tile position out of bounds in set_tile");
        }
        if self.tiles.contains_key(&pos) {
            bevy::log::error!("Tile position already occupied in set_tile");
        }
        self.tiles.insert(pos, entity);
    }

    pub fn remove_tile(&mut self, pos: IVec2) -> Option<Entity> {
        self.tiles.remove(&pos)
    }

    pub fn can_place(
        &self,
        kind: TetrominoKind,
        rotation: TetrominoRotation,
        new_pos: IVec2,
    ) -> bool {
        let shape = get_tetromino_shape(kind, rotation);
        for offset in shape.iter() {
            let pos = new_pos + offset;
            if pos.x < 0 || pos.x >= self.size.x as i32 || pos.y < 0 || self.get_tile(pos).is_some()
            {
                return false;
            }
        }

        true
    }

    pub fn place_tetromino(
        &mut self,
        commands: &mut Commands,
        tetromino: &Tetromino,
        tetromino_entity: Entity,
        asset_server: &Res<AssetServer>,
    ) {
        if self.can_place(tetromino.kind, tetromino.rotation, tetromino.pos) {
            for offset in get_tetromino_shape(tetromino.kind, tetromino.rotation) {
                let pos = tetromino.pos + offset;
                spawn_tile(commands, pos, tetromino.board_entity, true, asset_server);
            }
        } else {
            bevy::log::error!("Failed to place tetromino at {:?}", tetromino.pos);
        }

        commands.entity(tetromino_entity).despawn();
    }
}

pub fn spawn_board(
    commands: &mut Commands,
    size: UVec2,
    tile_size: UVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rec_size = (size * tile_size).as_vec2();
    commands.spawn((
        Name::new("Board"),
        Board::new(size, tile_size),
        Mesh2d(meshes.add(Rectangle::new(rec_size.x, rec_size.y))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_scale(vec3(4.0, 4.0, 4.0)),
    ));
}

fn spawn_next_tetromino(
    mut commands: Commands,
    boards: Query<(Entity, &Board)>,
    tetrominoes: Query<&Tetromino>,
) {
    let (board_entity, board) = try_unwrap!(
        boards.single(),
        "Expected one board when spawning tetromino"
    );

    for tetromino in tetrominoes.iter() {
        if tetromino.board_entity == board_entity {
            return; // Already a tetromino on the board
        }
    }

    let kind = match fastrand::i32(0..7) {
        0 => TetrominoKind::I,
        1 => TetrominoKind::J,
        2 => TetrominoKind::L,
        3 => TetrominoKind::O,
        4 => TetrominoKind::S,
        5 => TetrominoKind::T,
        _ => TetrominoKind::Z,
    }; // TODO replace with that one crate idk the name
    let pos = ivec2(4, board.size.y as i32);
    if !board.can_place(kind, 0, pos) {
        bevy::log::error!("Attempted to spawn tetromino at invalid position");
        return;
    }
    commands.entity(board_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Tetromino"),
            Tetromino::new(kind, pos, 80, board_entity),
        ));
    });
}
