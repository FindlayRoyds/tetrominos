use bevy::prelude::*;

#[derive(Copy, Clone)]
pub enum TetrominoType {
    // I,
    O,
    // T,
    // S,
    // Z,
    // J,
    L,
}

pub type TetrominoShape = [IVec2; 4];

pub const fn tetromino_shape(kind: TetrominoType) -> TetrominoShape {
    match kind {
        TetrominoType::O => [
            IVec2::new(0, 1),
            IVec2::new(0, 0),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
        TetrominoType::L => [
            IVec2::new(0, 2),
            IVec2::new(0, 1),
            IVec2::new(0, 0),
            IVec2::new(1, 0),
        ],
    }
}
