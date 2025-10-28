use bevy::prelude::*;

use crate::{
    board::{
        BoardUpdateSystems, TetrominoQueue, TetrominoQueueChanged,
        tetromino_data::{get_tetromino_display_offset, get_tetromino_shape},
        tile_assets::TileImages,
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
    pub length: u32,
}

impl QueueDisplay {
    fn update_display(
        &mut self,
        commands: &mut Commands,
        self_entity: Entity,
        queue: TetrominoQueue,
        tile_images: &Res<TileImages>,
        tiles: Query<(Entity, &Tile), With<QueueDisplayTile>>,
    ) {
        QueueDisplay::clear_display(commands, self_entity, tiles);

        for (i, kind) in queue.iter().take(self.length as usize).enumerate() {
            for offset in get_tetromino_shape(*kind, 0).iter() {
                let display_offset = get_tetromino_display_offset(*kind, 0, uvec2(4, 4));

                commands.spawn((
                    Name::new("QueueDisplayTile"),
                    Tile {
                        pos: (offset + ivec2(0, 4 * (self.length - 1 - i as u32) as i32)).as_vec2()
                            + display_offset,
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
    mut displays: Query<(Entity, &mut QueueDisplay)>,
    mut queue_changed_messages: MessageReader<TetrominoQueueChanged>,
    tiles: Query<(Entity, &Tile), With<QueueDisplayTile>>,
    tile_images: Res<TileImages>,
) {
    let messages: Vec<_> = queue_changed_messages.read().collect();

    for (display_entity, mut display) in displays.iter_mut() {
        for message in messages.iter().rev() {
            if display.board == message.board {
                display.update_display(
                    &mut commands,
                    display_entity,
                    message.new_queue.clone(),
                    &tile_images,
                    tiles,
                );
                break;
            }
        }
    }
}
