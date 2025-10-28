use bevy::prelude::*;

use crate::{
    board::{
        BoardUpdateSystems, TetrominoQueue, TetrominoQueueChanged,
        tetromino_data::get_tetromino_shape, tile_assets::TileImages,
    },
    tiles::{Tile, TileUpdateSystems},
};

pub struct QueueDisplayPlugin;

impl Plugin for QueueDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_queue_displays
                .after(BoardUpdateSystems)
                .before(TileUpdateSystems),
        );
    }
}

#[derive(Component)]
pub struct QueueDisplay {
    pub board: Entity,
}

impl QueueDisplay {
    fn update_display(
        commands: &mut Commands,
        self_entity: Entity,
        queue: TetrominoQueue,
        tile_images: &Res<TileImages>,
        tiles: Query<(Entity, &Tile), With<QueueDisplayTile>>,
    ) {
        QueueDisplay::clear_display(commands, self_entity, tiles);

        for (i, kind) in queue.iter().enumerate() {
            for offset in get_tetromino_shape(*kind, 0).iter() {
                commands.spawn((
                    Name::new("QueueDisplayTile"),
                    Tile {
                        pos: (offset + ivec2(i as i32 * -4, 0) + ivec2(1, 1)).as_vec2(),
                        tilemap: self_entity,
                    },
                    QueueDisplayTile,
                    ChildOf(self_entity),
                    Sprite::from_image(tile_images.0[kind].clone()),
                ));
            }
        }
    }

    fn clear_display(
        commands: &mut Commands,
        self_entity: Entity,
        tiles: Query<(Entity, &Tile), With<QueueDisplayTile>>,
    ) {
        for (tile_entity, tile) in tiles {
            if tile.tilemap == self_entity {
                commands.entity(tile_entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct QueueDisplayTile;

fn update_queue_displays(
    mut commands: Commands,
    displays: Query<(Entity, &QueueDisplay)>,
    mut queue_changed_messages: MessageReader<TetrominoQueueChanged>,
    tiles: Query<(Entity, &Tile), With<QueueDisplayTile>>,
    tile_images: Res<TileImages>,
) {
    for message in queue_changed_messages.read() {
        for (display_entity, display) in displays {
            if display.board == message.board {
                QueueDisplay::update_display(
                    &mut commands,
                    display_entity,
                    message.new_queue.clone(),
                    &tile_images,
                    tiles,
                );
            }
        }
    }
}
