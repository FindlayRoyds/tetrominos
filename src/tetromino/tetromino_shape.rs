use bevy::prelude::*;

#[derive(Copy, Clone)]
pub enum TetrominoKind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

pub type TetrominoShape = [IVec2; 4];
pub type TetrominoRotation = i32; // 0..4

const fn shape(kind: TetrominoKind) -> TetrominoShape {
    match kind {
        TetrominoKind::I => [ivec2(0, 0), ivec2(1, 0), ivec2(2, 0), ivec2(3, 0)],
        TetrominoKind::J => [ivec2(0, 0), ivec2(1, 0), ivec2(2, 0), ivec2(2, 1)],
        TetrominoKind::L => [ivec2(0, 0), ivec2(1, 0), ivec2(2, 0), ivec2(2, 1)],
        TetrominoKind::O => [ivec2(1, 0), ivec2(2, 0), ivec2(1, 1), ivec2(2, 1)],
        TetrominoKind::S => [ivec2(0, 0), ivec2(1, 0), ivec2(0, 1), ivec2(1, 1)],
        TetrominoKind::T => [ivec2(0, 0), ivec2(1, 0), ivec2(2, 0), ivec2(1, 1)],
        TetrominoKind::Z => [ivec2(1, 0), ivec2(2, 0), ivec2(0, 1), ivec2(1, 1)],
    }
}

// 0, 0 is in the center of the block at 0, 0
const fn pivot(kind: TetrominoKind) -> Vec2 {
    match kind {
        TetrominoKind::I => vec2(1.5, -0.5),
        TetrominoKind::J => vec2(1.0, 0.0),
        TetrominoKind::L => vec2(1.0, 0.0),
        TetrominoKind::O => vec2(0.5, 0.5),
        TetrominoKind::S => vec2(1.0, 0.0),
        TetrominoKind::T => vec2(1.0, 0.0),
        TetrominoKind::Z => vec2(1.0, 0.0),
    }
}

fn rotate_around_pivot(point: IVec2, pivot: Vec2, rotation: TetrominoRotation) -> IVec2 {
    let offset = point.as_vec2() - pivot;
    let rotated = match rotation.rem_euclid(4) {
        0 => vec2(offset.x, offset.y),
        1 => vec2(-offset.y, offset.x),
        2 => vec2(-offset.x, -offset.y),
        3 => vec2(offset.y, -offset.x),
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
