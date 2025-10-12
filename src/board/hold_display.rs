use bevy::prelude::*;

use crate::{
    board::{
        BoardUpdateSystems, HoldPieceChanged,
        tetromino_data::{TetrominoKind, get_tetromino_shape},
        tile_assets::TileImages,
    },
    tiles::{Tile, TileUpdateSystems},
};

pub struct HoldDisplayPlugin;

impl Plugin for HoldDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_hold_displays
                .after(BoardUpdateSystems)
                .before(TileUpdateSystems),
        );
    }
}

#[derive(Component)]
pub struct HoldDisplay {
    pub board: Entity,
}

impl HoldDisplay {
    fn update_display(
        commands: &mut Commands,
        self_entity: Entity,
        kind: TetrominoKind,
        tile_images: &Res<TileImages>,
        tiles: Query<(Entity, &Tile), With<HoldDisplayTile>>,
    ) {
        HoldDisplay::clear_display(commands, self_entity, tiles);

        for offset in get_tetromino_shape(kind, 0).iter() {
            commands.spawn((
                Name::new("HoldDisplayTile"),
                Tile {
                    pos: (offset + ivec2(1, 1)).as_vec2(),
                    tilemap: self_entity,
                },
                HoldDisplayTile,
                ChildOf(self_entity),
                Sprite::from_image(tile_images.0[&kind].clone()),
            ));
        }
    }

    fn clear_display(
        commands: &mut Commands,
        self_entity: Entity,
        tiles: Query<(Entity, &Tile), With<HoldDisplayTile>>,
    ) {
        for (tile_entity, tile) in tiles {
            if tile.tilemap == self_entity {
                commands.entity(tile_entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct HoldDisplayTile;

fn update_hold_displays(
    mut commands: Commands,
    displays: Query<(Entity, &HoldDisplay)>,
    mut hold_messages: MessageReader<HoldPieceChanged>,
    tiles: Query<(Entity, &Tile), With<HoldDisplayTile>>,
    tile_images: Res<TileImages>,
) {
    for message in hold_messages.read() {
        for (display_entity, display) in displays {
            if display.board == message.board {
                HoldDisplay::update_display(
                    &mut commands,
                    display_entity,
                    message.new_piece_kind,
                    &tile_images,
                    tiles,
                );
            }
        }
    }
}
