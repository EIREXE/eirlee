

use crate::{
    fighter::{
        animation::AnimKind,
        collision,
        state::{
            FighterState, FighterStateContext, FighterStateImpl, air,
            fall::FallState,
            ground::GroundedStateCommon,
            land::LandingState,
        },
    },
    input,
    math::vec::FGVec2,
};

#[derive(Debug, Clone, Hash)]
pub struct AirDodgeState {
    pub duration_counter: u32,
    pub direction: FGVec2,
}

impl FighterStateImpl for AirDodgeState {
    const NAME: &'static str = "AirDodge";

    fn check_interrupt(&self, state_context: &mut FighterStateContext) -> Option<FighterState> {
        if self.duration_counter >= state_context.fighter_manifest.attributes.air_dodge_duration {
            Some(FighterState::Fall(FallState))
        } else {
            None
        }
    }

    fn update(&mut self, state_context: &mut FighterStateContext) {
        self.duration_counter += 1;

        state_context.velocity.0 *= state_context.fighter_manifest.attributes.air_dodge_decay;

        air::apply_air_motion(
            state_context.velocity,
            state_context.translation,
            state_context.prev_translation,
        );
    }

    fn check_collision_interrupt(
        &mut self,
        state_context: &mut FighterStateContext,
    ) -> Option<FighterState> {
        if let Some(res) = collision::air_collide_with_stage(state_context) {
            // Adjust translation
            state_context.translation.0 = res.hit_position - state_context.ecb.get_bottom_point();
            let grounded_common = GroundedStateCommon {
                current_line_id: res.line_id,
            };
            Some(FighterState::Land(LandingState {
                grounded_common,
                duration_counter: 0,
            }))
        } else {
            None
        }
    }

    fn on_enter(&mut self, state_context: &mut FighterStateContext) {
        state_context.play_animation(AnimKind::AirDodge, false);

        state_context.velocity.0 =
            self.direction * state_context.fighter_manifest.attributes.air_dodge_velocity;
    }
}

pub fn check_input(state_context: &FighterStateContext) -> Option<FighterState> {
    state_context
        .input
        .has_command(input::FighterCommands::Shield)
        .then(|| {
            FighterState::AirDodge(AirDodgeState {
                duration_counter: 0,
                direction: state_context
                    .input
                    .get_last_frame()
                    .movement
                    .normalize_or_zero(),
            })
        })
}
