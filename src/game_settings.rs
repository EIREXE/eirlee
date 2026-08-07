use bevy::prelude::*;

pub struct FighterSettingsCommon {
    pub walk_speed_ease: f32,
    pub ground_max_horizontal_velocity: f32,
    pub ground_friction_over_walk_speed_multiplier: f32
}
pub struct InputSettingsCommon {
    pub stick_deadzone: f32,
    pub trigger_deadzone: f32,
    pub smash_input_reset_axis_threshold: f32,
    pub smash_input_axis_threshold: f32,
    pub run_stick_threshold: f32,
    pub smash_input_frame_threshold: u32,
}

impl Default for InputSettingsCommon {
    fn default() -> Self {
        Self {
            stick_deadzone: 0.28f32,
            trigger_deadzone: 0.3f32,
            smash_input_reset_axis_threshold: 0.25,
            smash_input_axis_threshold: 0.8,
            run_stick_threshold: 0.625,
            smash_input_frame_threshold: 2
        }
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub figher_common: FighterSettingsCommon,
    pub input_common: InputSettingsCommon
}