use fixed::types::I16F16;

use crate::{fighter::{attack::{AttackKind, update_active_hitbox_list}, state::{FighterState, FighterStateImpl, ground::GroundedStateCommon, wait::WaitState}}, math::int::FGi32};

#[derive(Clone, Debug, Hash)]
pub struct GroundAttackState {
    pub attack_kind: AttackKind,
    pub grounded_common: GroundedStateCommon,
}

impl FighterStateImpl for GroundAttackState {
    const NAME: &'static str = "GroundAttack";

    fn check_interrupt(&self, state_context: &mut super::FighterStateContext) -> Option<FighterState> {
        if state_context.is_current_animation_finished() {
            Some(FighterState::Wait(WaitState {
                grounded_common: self.grounded_common.clone()
            }))
        } else {
            None
        }
    }

    fn update(&mut self, state_context: &mut super::FighterStateContext) {
        let script = state_context.attack_scripts.get(&state_context.attack_script_assets.scripts[&self.attack_kind]);
        if let Some(script) = script { 
            let animation_frame = state_context.get_current_animation_frame();
            update_active_hitbox_list(script, &mut state_context.hitboxes.active_hitboxes, animation_frame);
        }
    }
    
    fn on_enter(&mut self, state_context: &mut super::FighterStateContext) {
        state_context.play_animation(self.attack_kind.get_animation(), false);
        state_context.hitboxes.attack_script = Some(state_context.attack_script_assets.scripts[&self.attack_kind].clone());
        state_context.hitboxes.active_hitboxes.clear();
    }
    
    fn on_exit(&mut self, state_context: &mut super::FighterStateContext) {
        state_context.hitboxes.clear();
    }
    
    fn check_collision_interrupt(
        &mut self,
        _state_context: &mut super::FighterStateContext,
    ) -> Option<FighterState> {
        None
    }
}

pub fn check_interrupt(state_context: &super::FighterStateContext, grounded_common: &GroundedStateCommon) -> Option<FighterState> {
    if state_context.input.has_command(crate::input::FighterCommands::Attack) {

        let attack_kind = if state_context.input.get_last_frame().movement.y > FGi32::ZERO {
            AttackKind::UpTilt
        } else {
            AttackKind::Jab
        };

        Some(FighterState::GroundAttack(GroundAttackState {
            attack_kind,
            grounded_common: grounded_common.clone()
        }))
    } else {
        None
    }
}