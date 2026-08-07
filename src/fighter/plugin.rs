use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use crate::fighter::state::StateNameDebug;
use crate::fighter::{FighterVelocity, states};
use crate::fighter::{
    apply_motion, collide_fighter_with_scene, integrate_gravity, state::state_interrupt_system,
};
use crate::input::player::FighterInput;

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
                    state_interrupt_system::<states::run::RunState>,
                    states::wait::wait_update,
                    states::walk::walk_update,
                    states::dash::dash_update,
                    states::run::run_update,
                    integrate_gravity,
                    apply_motion,
                    collide_fighter_with_scene,
                )
                    .chain(),
            )
            .add_observer(states::dash::on_insert_dash)
            .add_systems(EguiPrimaryContextPass, fighter_debug);
    }
}

fn fighter_debug(mut contexts: EguiContexts, query: Query<(&FighterVelocity, &FighterInput, &StateNameDebug)>) -> Result {
    for (vel, input, name) in query {
        egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
            ui.label(format!("State: {}", name.0));
            ui.label(format!("{:?}", vel));
            ui.label(format!("{:?}", input.get_last_frame()));
        });
    }

    Ok(())
}
