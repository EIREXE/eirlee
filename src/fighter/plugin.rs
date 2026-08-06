use bevy::prelude::*;
use crate::fighter::states;
use crate::fighter::{
    FighterECB, FighterTranslation, FighterVelocity, apply_motion, collide_fighter_with_scene, integrate_gravity, state::state_interrupt_system,
};

#[derive(Default)]
pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, super::debug_draw_ecb)
            .add_systems(
                FixedUpdate,
                (
                    state_interrupt_system::<states::wait::WaitState>,
                    state_interrupt_system::<states::walk::WalkState>,
                    state_interrupt_system::<states::dash::DashState>,
                    states::walk::walk_update,
                    states::dash::dash_update,
                    integrate_gravity,
                    apply_motion,
                    collide_fighter_with_scene,
                )
                    .chain(),
            )
            .add_observer(states::dash::on_insert_dash);
    }
}
