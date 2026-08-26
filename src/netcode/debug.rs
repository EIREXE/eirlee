//! Network stats overlay.

use bevy::prelude::*;
use bevy_egui::prelude::*;
use bevy_ggrs::prelude::*;

use crate::netcode::GGRSCfg;

pub fn network_debug(mut contexts: EguiContexts, session: Option<Res<Session<GGRSCfg>>>) -> Result {
    if let Some(sess) = session {
        match sess.as_ref() {
            Session::P2P(s) => {
                let num_players = s.num_players();
                for i in 0..num_players {
                    egui::Window::new(format!("Player {}", i)).show(contexts.ctx_mut()?, |ui| {
                        if let Ok(stats) = s.network_stats(i) {
                            ui.label(format!("{:?}", stats));
                        }
                    });
                }
            }
            _ => {}
        }
    }

    Ok(())
}
