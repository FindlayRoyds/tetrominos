use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    ShiftLeft,
    ShiftRight,
    SoftDrop,
    HardDrop,
    RotateLeft,
    RotateRight,
    Hold,
}

pub fn get_board_input_map() -> InputMap<Action> {
    use Action::*;
    let mut input_map = InputMap::default();

    input_map.insert(ShiftLeft, KeyCode::KeyA);
    input_map.insert(ShiftLeft, KeyCode::ArrowLeft);

    input_map.insert(ShiftRight, KeyCode::KeyD);
    input_map.insert(ShiftRight, KeyCode::ArrowRight);

    input_map.insert(SoftDrop, KeyCode::KeyS);
    input_map.insert(SoftDrop, KeyCode::ArrowDown);

    input_map.insert(HardDrop, KeyCode::Space);

    input_map.insert(RotateRight, KeyCode::KeyW);
    input_map.insert(RotateRight, KeyCode::ArrowUp);

    input_map.insert(Hold, KeyCode::KeyC);

    input_map
}
