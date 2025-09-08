use bevy::prelude::*;

use crate::{
    tetrominoes::{Tetromino, TetrominoKind, TetrominoRotation, get_tetromino_shape},
    tile::{Tile, spawn_tile},
    try_unwrap,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_next_tetromino.in_set(TetrominoSpawning),
                clear_lines.in_set(LineClearing),
            ),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TetrominoSpawning;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct LineClearing;

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
        }
    }

    pub fn get_tile(
        &self,
        board_entity: Entity,
        pos: IVec2,
        tiles: Query<(Entity, &Tile)>,
    ) -> Option<Entity> {
        tiles
            .iter()
            .find(|(_, tile)| tile.placed && tile.board_entity == board_entity && tile.pos == pos)
            .map(|(entity, _)| entity)
    }

    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }

    pub fn can_place(
        &self,
        board_entity: Entity,
        tiles: Query<(Entity, &Tile)>,
        kind: TetrominoKind,
        rotation: TetrominoRotation,
        new_pos: IVec2,
    ) -> bool {
        let shape = get_tetromino_shape(kind, rotation);
        for offset in shape.iter() {
            let pos = new_pos + offset;
            if pos.x < 0
                || pos.x >= self.size.x as i32
                || pos.y < 0
                || self.get_tile(board_entity, pos, tiles).is_some()
            {
                return false;
            }
        }

        true
    }

    pub fn place_tetromino(
        &mut self,
        commands: &mut Commands,
        board_entity: Entity,
        tiles: Query<(Entity, &Tile)>,
        tetromino: &Tetromino,
        tetromino_entity: Entity,
        asset_server: &Res<AssetServer>,
    ) {
        if self.can_place(
            board_entity,
            tiles,
            tetromino.kind,
            tetromino.rotation,
            tetromino.pos,
        ) {
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
    tiles: Query<(Entity, &Tile)>,
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
    let pos = ivec2(4, board.size.y as i32 + 1);
    if !board.can_place(board_entity, tiles, kind, 0, pos) {
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

fn clear_lines(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board)>,
    mut tile_queries: ParamSet<(Query<(Entity, &Tile)>, Query<(Entity, &mut Tile)>)>,
) {
    for (board_entity, board) in boards.iter_mut() {
        let mut num_cleared_lines = 0;

        for y in 0..board.size.y as i32 {
            let mut tile_entities: Vec<(Entity, IVec2)> = vec![];
            for x in 0..board.size.x as i32 {
                if let Some(tile_entity) =
                    board.get_tile(board_entity, ivec2(x, y), tile_queries.p0())
                {
                    tile_entities.push((tile_entity, ivec2(x, y)));
                }
            }

            if tile_entities.len() == board.size.x as usize {
                num_cleared_lines += 1;
                for (tile_entity, _) in tile_entities {
                    if let Ok(mut tile_commands) = commands.get_entity(tile_entity) {
                        tile_commands.despawn();
                    }
                }
            } else if num_cleared_lines > 0 {
                for (tile_entity, _) in tile_entities {
                    if let Ok((_, mut tile)) = tile_queries.p1().get_mut(tile_entity) {
                        tile.pos -= ivec2(0, num_cleared_lines);
                    }
                }
            }
        }
    }
}
