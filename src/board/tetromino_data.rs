use bevy::prelude::*;
use strum_macros::{EnumCount, EnumIter};

#[derive(EnumIter, EnumCount, Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
        TetrominoKind::I => [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(2, 0)],
        TetrominoKind::J => [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(-1, 1)],
        TetrominoKind::L => [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(1, 1)],
        TetrominoKind::O => [ivec2(0, 0), ivec2(1, 0), ivec2(0, 1), ivec2(1, 1)],
        TetrominoKind::S => [ivec2(-1, 0), ivec2(0, 0), ivec2(0, 1), ivec2(1, 1)],
        TetrominoKind::T => [ivec2(-1, 0), ivec2(0, 0), ivec2(1, 0), ivec2(0, 1)],
        TetrominoKind::Z => [ivec2(0, 0), ivec2(1, 0), ivec2(-1, 1), ivec2(0, 1)],
    }
}

fn rotate(point: IVec2, rotation: TetrominoRotation) -> IVec2 {
    match rotation % 4 {
        0 => ivec2(point.x, point.y),
        1 => ivec2(point.y, -point.x),
        2 => ivec2(-point.x, -point.y),
        3 => ivec2(-point.y, point.x),
        _ => unreachable!(),
    }
}

pub fn get_tetromino_shape(kind: TetrominoKind, rotation: TetrominoRotation) -> TetrominoShape {
    let mut rotated_shape = [IVec2::ZERO; 4];
    for (i, point) in shape(kind).iter().enumerate() {
        rotated_shape[i] = rotate(*point, rotation);
    }
    rotated_shape
}

fn offsets(kind: TetrominoKind, rotation: TetrominoRotation) -> Vec<IVec2> {
    const R: TetrominoRotation = 1;
    const L: TetrominoRotation = 3;

    let offsets = match kind {
        TetrominoKind::O => match rotation % 4 {
            0 => vec![(0, 0)],
            R => vec![(0, -1)],
            2 => vec![(-1, -1)],
            L => vec![(-1, 0)],
            _ => unreachable!(),
        },
        TetrominoKind::I => match rotation % 4 {
            0 => vec![(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
            R => vec![(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
            2 => vec![(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
            L => vec![(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
            _ => unreachable!(),
        },
        _ => match rotation % 4 {
            0 => vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
            R => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            2 => vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
            L => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            _ => unreachable!(),
        },
    };

    offsets
        .iter()
        .map(|offset| ivec2(offset.0, offset.1))
        .collect()
}

pub fn get_tetromino_wall_kicks(
    original_rotation: TetrominoRotation,
    new_rotation: TetrominoRotation,
    kind: TetrominoKind,
) -> Vec<IVec2> {
    if (new_rotation - original_rotation).abs() == 2 {
        bevy::log::error!("Invalid tetromino rotation (2)")
    }

    let original_offsets = offsets(kind, original_rotation);
    let new_offsets = offsets(kind, new_rotation);

    original_offsets
        .iter()
        .zip(new_offsets.iter())
        .map(|(o, n)| o - n)
        .collect()
}

pub const fn get_tetromino_color(kind: TetrominoKind) -> &'static str {
    match kind {
        TetrominoKind::I => "blue",
        TetrominoKind::J => "pink",
        TetrominoKind::L => "orange",
        TetrominoKind::O => "yellow",
        TetrominoKind::S => "red",
        TetrominoKind::T => "purple",
        TetrominoKind::Z => "green",
    }
}
