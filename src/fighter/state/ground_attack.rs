use crate::fighter::{attack::{AttackKind, update_active_hitbox_list}, state::{FighterState, FighterStateImpl, ground::GroundedStateCommon, wait::WaitState}};

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
}

pub fn check_interrupt(state_context: &super::FighterStateContext, grounded_common: &GroundedStateCommon) -> Option<FighterState> {
    if state_context.input.has_command(crate::input::FighterCommands::Attack) {
        Some(FighterState::GroundAttack(GroundAttackState {
            attack_kind: AttackKind::Jab,
            grounded_common: grounded_common.clone()
        }))
    } else {
        None
    }
}