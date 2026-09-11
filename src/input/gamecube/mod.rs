pub mod bridge;
pub mod poll;

pub use bridge::{GC_PORT_COUNT, GcPort, GcPorts};
pub use poll::{GcAdapterSnapshot, GcPortState};

use bevy::input::InputSystems;
use bevy::input::gamepad::{gamepad_connection_system, gamepad_event_processing_system};
use bevy::prelude::*;

#[derive(Default)]
pub struct GamecubeAdapterPlugin;

impl Plugin for GamecubeAdapterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GcPorts>();

        let Ok(snapshot) = poll::spawn_poll_thread() else {
            // No adapter connected; leave GcPorts empty and skip wiring the
            // polling systems.
            return;
        };

        app.insert_resource(snapshot).add_systems(
            PreUpdate,
            (
                bridge::gc_connection_system.before(gamepad_connection_system),
                bridge::gc_event_system
                    .after(gamepad_connection_system)
                    .before(gamepad_event_processing_system),
            )
                .in_set(InputSystems),
        );
    }
}
