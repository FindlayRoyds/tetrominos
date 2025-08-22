use bevy::prelude::*;

pub struct TetrominoPlugin;

impl Plugin for TetrominoPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, update_tetromino_positions);
    }
}

#[derive(Component)]
pub struct Tetromino {
    pub shape: Vec<IVec2>,
    pub position: IVec2,
    pub board: Entity,
}
