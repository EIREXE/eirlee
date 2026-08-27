//! Turns [`GcAdapterSnapshot`] port states into real Bevy `Gamepad`
//! entities, the same way `bevy_gilrs` turns OS gamepad events into them.

use bevy::input::gamepad::{
    GamepadAxis, GamepadButton, GamepadConnection, GamepadConnectionEvent, GamepadSettings,
    RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent,
};
use bevy::prelude::*;

use super::poll::{GcAdapterSnapshot, GcPortState};

/// GameCube adapters report 4 ports.
pub const GC_PORT_COUNT: usize = 4;

/// USB vendor/product ID of the official adapter, reported on the `Gamepad`
/// component so it shows up the same way a real USB gamepad would.
const GC_ADAPTER_VID: u16 = 0x057e;
const GC_ADAPTER_PID: u16 = 0x0337;

/// Marks a `Gamepad` entity as backed by a specific GameCube adapter port
/// (0-3), and is the join key [`crate::input::gamepad::sample_gamepad`]
/// uses to map GGRS handles to ports.
#[derive(Component, Debug, Clone, Copy)]
pub struct GcPort(pub u8);

/// Tracks which entity (if any) represents each of the 4 adapter ports,
/// and the last snapshot seen, so both systems below can diff instead of
/// re-deriving state every frame.
#[derive(Resource, Default)]
pub struct GcPorts {
    pub entities: [Option<Entity>; GC_PORT_COUNT],
    last: [GcPortState; GC_PORT_COUNT],
}

fn passthrough_gamepad_settings() -> GamepadSettings {
    let mut settings = GamepadSettings::default();
    for axis in GamepadAxis::all() {
        let axis_settings = settings.axis_settings.entry(axis).or_default();
        // Order matters: each setter rejects a value that would cross the
        // *other* bound it's paired with, so widen the live zones before
        // shrinking the dead zones down to zero.
        axis_settings.set_livezone_lowerbound(-1.0);
        axis_settings.set_livezone_upperbound(1.0);
        axis_settings.set_deadzone_lowerbound(0.0);
        axis_settings.set_deadzone_upperbound(0.0);
        axis_settings.set_threshold(0.0);
    }
    for button in GamepadButton::all() {
        let button_axis_settings = settings.button_axis_settings.entry(button).or_default();
        button_axis_settings.low = 0.0;
        button_axis_settings.high = 1.0;
        button_axis_settings.threshold = 0.0;
    }
    settings
}

/// Spawns/despawns `Gamepad` entities to match which adapter ports are
/// connected. Must run before `gamepad_connection_system` so the entity it
/// spawns already exists by the time Bevy inserts the `Gamepad` component
/// on it (mirrors what `bevy_gilrs::gilrs_event_system` does).
pub fn gc_connection_system(
    mut commands: Commands,
    snapshot: Res<GcAdapterSnapshot>,
    mut ports: ResMut<GcPorts>,
    mut connection_events: MessageWriter<GamepadConnectionEvent>,
) {
    let current = snapshot.read();
    let ports = &mut *ports;

    for i in 0..GC_PORT_COUNT {
        let was_connected = ports.last[i].connected;
        let is_connected = current[i].connected;

        if is_connected && !was_connected {
            let entity = ports.entities[i].unwrap_or_else(|| {
                let entity = commands
                    .spawn((GcPort(i as u8), passthrough_gamepad_settings()))
                    .id();
                ports.entities[i] = Some(entity);
                entity
            });
            connection_events.write(GamepadConnectionEvent::new(
                entity,
                GamepadConnection::Connected {
                    name: format!("GameCube Controller (Port {})", i + 1),
                    vendor_id: Some(GC_ADAPTER_VID),
                    product_id: Some(GC_ADAPTER_PID),
                },
            ));
        } else if !is_connected && was_connected {
            if let Some(entity) = ports.entities[i] {
                connection_events.write(GamepadConnectionEvent::new(
                    entity,
                    GamepadConnection::Disconnected,
                ));
            }
        }
    }
}

/// Converts a GC unsigned trigger byte to the 0.0..=1.0 range
/// `GamepadButton` analog values use.
fn unsigned_axis_float(raw: u8) -> f32 {
    (raw as f32) / 255.0
}

/// Diffs the current snapshot against last frame's and writes
/// `RawGamepadEvent`s for anything that changed, which
/// `gamepad_event_processing_system` turns into live `Gamepad` component
/// state. Must run after `gc_connection_system` (so newly-connected ports
/// have an entity/`GamepadSettings`) and before
/// `gamepad_event_processing_system`.
pub fn gc_event_system(
    snapshot: Res<GcAdapterSnapshot>,
    mut ports: ResMut<GcPorts>,
    mut raw_events: MessageWriter<RawGamepadEvent>,
    mut axis_events: MessageWriter<RawGamepadAxisChangedEvent>,
    mut button_events: MessageWriter<RawGamepadButtonChangedEvent>,
) {
    let current = snapshot.read();

    for i in 0..GC_PORT_COUNT {
        let Some(entity) = ports.entities[i] else {
            continue;
        };
        let prev = &ports.last[i];
        let now = &current[i];
        if !now.connected {
            continue;
        }

        macro_rules! send_axis {
            ($axis:expr, $value:expr) => {{
                let event = RawGamepadAxisChangedEvent::new(entity, $axis, $value);
                axis_events.write(event);
                raw_events.write(RawGamepadEvent::Axis(event));
            }};
        }
        macro_rules! send_button {
            ($button:expr, $value:expr) => {{
                let event = RawGamepadButtonChangedEvent::new(entity, $button, $value);
                button_events.write(event);
                raw_events.write(RawGamepadEvent::Button(event));
            }};
        }

        if prev.left_stick != now.left_stick {
            send_axis!(GamepadAxis::LeftStickX, now.left_stick.0);
            send_axis!(GamepadAxis::LeftStickY, now.left_stick.1);
        }
        if prev.right_stick != now.right_stick {
            send_axis!(
                GamepadAxis::RightStickX,
                now.right_stick.0
            );
            send_axis!(
                GamepadAxis::RightStickY,
                now.right_stick.1
            );
        }
        if prev.triggers != now.triggers {
            // The analog triggers are represented as button-axis values, as we have no GamepadAxis to use.
            send_button!(GamepadButton::LeftTrigger2, unsigned_axis_float(now.triggers.0));
            send_button!(GamepadButton::RightTrigger2, unsigned_axis_float(now.triggers.1));
        }

        macro_rules! digital {
            ($field:ident, $button:expr) => {
                if prev.$field != now.$field {
                    send_button!($button, if now.$field { 1.0 } else { 0.0 });
                }
            };
        }
        digital!(a, GamepadButton::South);
        digital!(b, GamepadButton::West);
        digital!(x, GamepadButton::East);
        digital!(y, GamepadButton::North);
        digital!(z, GamepadButton::Z);
        digital!(start, GamepadButton::Start);
        digital!(dpad_up, GamepadButton::DPadUp);
        digital!(dpad_down, GamepadButton::DPadDown);
        digital!(dpad_left, GamepadButton::DPadLeft);
        digital!(dpad_right, GamepadButton::DPadRight);
        digital!(left_trigger_digital, GamepadButton::LeftTrigger);
        digital!(right_trigger_digital, GamepadButton::RightTrigger);
    }

    ports.last = current;
}
