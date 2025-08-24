use bevy::prelude::*;

use crate::tile::{Board, Tile, spawn_tile};

pub struct TetrominoPlugin;

impl Plugin for TetrominoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (clear_tiles, apply_gravity, update_positions, spawn_tiles)
                .chain()
                .in_set(TetrominoUpdates),
        );
    }
}

pub type TetrominoShape = Vec<IVec2>;

#[derive(Component)]
pub struct Tetromino {
    pub shape: TetrominoShape,
    pub pos: IVec2,
    pub board_entity: Entity,

    /// How far below the actual position the tetromino is (used for movements less than a block)
    pub vertical_offset: f32,
}

impl Tetromino {
    pub fn new(shape: Vec<IVec2>, pos: IVec2, board_entity: Entity) -> Self {
        Self {
            shape,
            pos,
            board_entity,
            vertical_offset: 0.0,
        }
    }
}

#[derive(Component)]
pub struct TetrominoTile;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
pub struct TetrominoUpdates;

pub fn is_tetromino_pos_valid(shape: TetrominoShape, new_pos: IVec2, board: &Board) -> bool {
    for offset in shape.iter() {
        let pos = new_pos + offset;
        if pos.x < 0 || pos.x >= board.size.x as i32 || pos.y < 0 || board.get_tile(pos).is_some() {
            return false;
        }
    }

    return true;
}

fn place_tetromino(
    commands: &mut Commands,
    tetromino: &Tetromino,
    tetromino_entity: Entity,
    asset_server: &Res<AssetServer>,
) {
    bevy::log::info!("Placing tetromino at {:?}", tetromino.pos);
    for offset in tetromino.shape.iter() {
        let pos = tetromino.pos + offset;
        spawn_tile(commands, pos, tetromino.board_entity, true, asset_server);
    }

    commands.entity(tetromino_entity).despawn();
}

fn clear_tiles(mut commands: Commands, tiles: Query<Entity, (With<Tile>, With<TetrominoTile>)>) {
    for tile_entity in tiles.iter() {
        commands
            .get_entity(tile_entity)
            .expect("Failed to get tile entity")
            .despawn();
    }
}

fn spawn_tiles(
    mut commands: Commands,
    tetrominos: Query<&Tetromino>,
    asset_server: Res<AssetServer>,
) {
    for tetromino in tetrominos.iter() {
        let board_entity = tetromino.board_entity;
        for offset in tetromino.shape.iter() {
            let pos = tetromino.pos + offset;
            let tile_entity = spawn_tile(&mut commands, pos, board_entity, false, &asset_server);
            commands
                .get_entity(tile_entity)
                .expect("Failed to get tile entity")
                .insert(TetrominoTile);
        }
    }
}

fn apply_gravity(mut tetrominos: Query<&mut Tetromino>) {
    for mut tetromino in tetrominos.iter_mut() {
        tetromino.vertical_offset -= 0.1;
    }
}

fn update_positions(
    mut commands: Commands,
    mut tetrominos: Query<(Entity, &mut Tetromino)>,
    boards: Query<&Board>,
    asset_server: Res<AssetServer>,
) {
    for (tetromino_entity, mut tetromino) in tetrominos.iter_mut() {
        let board = boards.get(tetromino.board_entity).expect("Board not found");

        let vertical_change = tetromino.vertical_offset.floor() as i32;
        let new_pos = tetromino.pos + IVec2::new(0, vertical_change);
        if !is_tetromino_pos_valid(tetromino.shape.clone(), new_pos, board) {
            tetromino.vertical_offset = 0.0; // Not needed currently, could be useful for lock delay
            if is_tetromino_pos_valid(tetromino.shape.clone(), tetromino.pos, board) {
                place_tetromino(&mut commands, &tetromino, tetromino_entity, &asset_server);
            } else {
                bevy::log::warn!("Failed to place tetromino at {:?}", tetromino.pos);
                commands.entity(tetromino_entity).despawn();
            }
            continue;
        }
        tetromino.pos = new_pos;
        tetromino.vertical_offset -= vertical_change as f32;
    }
}
