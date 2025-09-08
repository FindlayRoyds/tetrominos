use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    tetrominoes::{Tetromino, TetrominoKind, TetrominoRotation, get_tetromino_shape},
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
    pub drop_delay: i32,
    pub drop_delay_counter: i32,

    // Inputs
    pub shift: i32,
    pub auto_shift: i32,
    pub auto_shift_delay: i32,
    pub soft_drop: bool,
    pub hard_drop: bool,

    tiles: HashMap<IVec2, Entity>,
}

impl Board {
    pub fn new(size: UVec2, tile_size: UVec2, drop_delay: i32) -> Self {
        Self {
            size,
            tile_size,
            drop_delay,
            drop_delay_counter: drop_delay,

            shift: 0,
            auto_shift_delay: 0, // TODO set to config value, fine for now though
            auto_shift: 0,
            soft_drop: false,
            hard_drop: false,

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
}

pub fn spawn_board(
    commands: &mut Commands,
    size: UVec2,
    tile_size: UVec2,
    drop_delay: i32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rec_size = (size * tile_size).as_vec2();
    commands.spawn((
        Name::new("Board"),
        Board::new(size, tile_size, drop_delay),
        Mesh2d(meshes.add(Rectangle::new(rec_size.x, rec_size.y))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_scale(vec3(4.0, 4.0, 4.0)),
    ));
}

fn spawn_next_tetromino(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board)>,
    tetrominoes: Query<&Tetromino>,
) {
    let (board_entity, mut board) = try_unwrap!(
        boards.single_mut(),
        "Expected one board when spawning tetromino"
    );

    for tetromino in tetrominoes.iter() {
        if tetromino.board_entity == board_entity {
            return; // Already a tetromino on the board
        }
    }

    if board.drop_delay_counter > 0 {
        board.drop_delay_counter -= 1;
        return;
    }
    board.drop_delay_counter = board.drop_delay;

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
