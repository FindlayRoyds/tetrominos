use bevy::{platform::collections::HashMap, prelude::*};
use strum::IntoEnumIterator;

use crate::board::tetromino_data::{TetrominoKind, get_tetromino_color};

pub struct TileAssets;

impl Plugin for TileAssets {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup); // TODO make this not prestartup. Use a loading state ideally
    }
}

#[derive(Resource, Default)]
pub struct TileImages(pub HashMap<TetrominoKind, Handle<Image>>);

#[derive(Resource, Default)]
pub struct TileOutlineImages(pub HashMap<TetrominoKind, Handle<Image>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut images = TileImages::default();
    let mut outline_images = TileOutlineImages::default();

    for kind in TetrominoKind::iter() {
        let color_str = get_tetromino_color(kind);

        let image: Handle<Image> = asset_server.load(format!("tiles/tile_{}.png", color_str));
        images.0.insert(kind, image);
        let outline_image: Handle<Image> =
            asset_server.load(format!("tiles/outline_{}.png", color_str));
        outline_images.0.insert(kind, outline_image);
    }

    commands.insert_resource(images);
    commands.insert_resource(outline_images);
}
