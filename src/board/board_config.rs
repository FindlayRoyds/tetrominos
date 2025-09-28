use bevy::prelude::*;

#[derive(Component)]
pub struct BoardConfig {
    pub auto_shift_delay: i32,
    pub auto_shift_speed: f32,

    pub stationary_lock_delay: i32,
    pub lock_delay: i32,

    pub soft_drop_speed: f32,

    pub line_clear_fade_time: i32,
    pub line_clear_delay: i32,
    pub line_clear_horizontal_delay: i32,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            auto_shift_delay: 8,
            auto_shift_speed: 0.25,

            stationary_lock_delay: 40,
            lock_delay: 200,

            soft_drop_speed: 0.25,

            line_clear_fade_time: 5,
            line_clear_delay: 10,
            line_clear_horizontal_delay: 2,
        }
    }
}
