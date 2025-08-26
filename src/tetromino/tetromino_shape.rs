use bevy::prelude::*;

#[derive(Copy, Clone)]
pub enum TetrominoKind {
    // I,
    O,
    // T,
    // S,
    // Z,
    // J,
    L,
}

pub type TetrominoShape = [IVec2; 4];
pub type TetrominoRotation = i32; // 0..4

const fn shape(kind: TetrominoKind) -> TetrominoShape {
    match kind {
        TetrominoKind::O => [
            IVec2::new(0, 1),
            IVec2::new(0, 0),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
        TetrominoKind::L => [
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
            IVec2::new(2, 1),
        ],
    }
}

const fn pivot(kind: TetrominoKind) -> Vec2 {
    match kind {
        TetrominoKind::O => Vec2::new(0.5, 0.5),
        TetrominoKind::L => Vec2::new(1.0, 0.0),
    }
}

fn rotate_around_pivot(point: IVec2, pivot: Vec2, rotation: TetrominoRotation) -> IVec2 {
    let offset = point.as_vec2() - pivot;
    let rotated = match rotation.rem_euclid(4) {
        0 => Vec2::new(offset.x, offset.y),
        1 => Vec2::new(-offset.y, offset.x),
        2 => Vec2::new(-offset.x, -offset.y),
        3 => Vec2::new(offset.y, -offset.x),
        _ => unreachable!(),
    };
    return (rotated + pivot).round().as_ivec2();
}

pub fn get_tetromino_shape(kind: TetrominoKind, rotation: TetrominoRotation) -> TetrominoShape {
    let pivot = pivot(kind);
    let mut rotated_shape = [IVec2::ZERO; 4];
    for (i, point) in shape(kind).iter().enumerate() {
        rotated_shape[i] = rotate_around_pivot(*point, pivot, rotation);
    }
    return rotated_shape;
}
