use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use gc_adapter_neo::{Controller, GcAdapter};
#[derive(Clone, Copy, PartialEq, Default)]
pub struct GcPortState {
    pub connected: bool,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub start: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub left_trigger_digital: bool,
    pub right_trigger_digital: bool,
    pub left_stick: (f32, f32),
    pub right_stick: (f32, f32),
    /// Raw trigger bytes, 0 is released, 255 is fully pressed.
    pub triggers: (u8, u8),
}

impl GcPortState {
    fn make_port_state(c: &Controller, cal: &gc_adapter_neo::PortCalibration) -> Self {
        if !c.connected() {
            return Self::default();
        }
        let b = &c.buttons;
        Self {
            connected: true,
            a: b.a(),
            b: b.b(),
            x: b.x(),
            y: b.y(),
            z: b.z(),
            start: b.start(),
            dpad_up: b.dpad_up(),
            dpad_down: b.dpad_down(),
            dpad_left: b.dpad_left(),
            dpad_right: b.dpad_right(),
            left_trigger_digital: b.left_trigger(),
            right_trigger_digital: b.right_trigger(),
            left_stick: cal.left_stick(&c),
            right_stick: cal.right_stick(&c),
            triggers: (c.triggers.left.raw(), c.triggers.right.raw()),
        }
    }
}

/// Shared, `Send + Sync` handle to the latest snapshot from the poll
/// thread. Safe to store as an ordinary [`Resource`] (no `NonSend` needed)
/// because the `GcAdapter` itself never leaves its thread.
#[derive(Resource, Clone)]
pub struct GcAdapterSnapshot {
    ports: Arc<Mutex<[GcPortState; 4]>>,
    shutdown: Arc<AtomicBool>,
}

impl GcAdapterSnapshot {
    pub fn read(&self) -> [GcPortState; 4] {
        *self.ports.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for GcAdapterSnapshot {
    fn drop(&mut self) {
        // Only actually signals shutdown once the last clone (this resource
        // is never cloned in practice, but be defensive) is dropped.
        if Arc::strong_count(&self.shutdown) <= 1 {
            self.shutdown.store(true, Ordering::Relaxed);
        }
    }
}

/// Connects to the first GameCube adapter found on the USB bus and starts
/// polling it on a dedicated thread. Returns `None` (after logging a
/// warning) if no adapter is plugged in
pub fn spawn_poll_thread() -> Result<GcAdapterSnapshot, gc_adapter_neo::UsbError> {
    let mut adapter = match GcAdapter::from_usb()? {
        Some(adapter) => adapter,
        None => {
            warn!("No GameCube Controller Adapter found on USB; gamepad input disabled.");
            return Err(gc_adapter_neo::UsbError::Disconnected);
        }
    };

    let ports = Arc::new(Mutex::new([GcPortState::default(); 4]));
    let shutdown = Arc::new(AtomicBool::new(false));

    let thread_ports = ports.clone();
    let thread_shutdown = shutdown.clone();
    std::thread::Builder::new()
        .name("gc-adapter-poll".into())
        .spawn(move || {
            // Flush whatever the adapter had buffered before we started
            if let Err(payload) =
                panic::catch_unwind(AssertUnwindSafe(|| adapter.refresh_inputs()))
            {
                error!(
                    "GameCube Controller Adapter USB refresh panicked on startup: {payload:?}"
                );
            }

            #[cfg(debug_assertions)]
            let mut rate_window_start = std::time::Instant::now();
            #[cfg(debug_assertions)]
            let mut rate_window_count: u32 = 0;

            while !thread_shutdown.load(Ordering::Relaxed) {
                let result =
                    panic::catch_unwind(AssertUnwindSafe(|| adapter.read_controllers()));

                let controllers = match result {
                    Ok(Ok(controllers)) => controllers,
                    // Ignore this, for some reason we get a bunch of these when we start reading
                    Ok(Err(
                        gc_adapter_neo::AdapterError::Parse(_)
                        | gc_adapter_neo::AdapterError::ShortRead(_),
                    )) => continue,
                    Ok(Err(gc_adapter_neo::AdapterError::Usb(err))) => {
                        error!(
                            "GameCube Controller Adapter USB read failed ({err:?}); stopping \
                             polling. Gamepad input will stop updating until the game is \
                             restarted."
                        );
                        break;
                    }
                    Err(payload) => {
                        error!(
                            "GameCube Controller Adapter USB read panicked: {payload:?}; \
                             stopping polling. Gamepad input will stop updating until the \
                             game is restarted."
                        );
                        break;
                    }
                };

                let mut states = [GcPortState::default(); 4];
                for i in 0..4 {
                    states[i] = GcPortState::make_port_state(&controllers[i], adapter.calibration(i));
                }
                *thread_ports.lock().unwrap_or_else(|e| e.into_inner()) = states;

                #[cfg(debug_assertions)]
                {
                    rate_window_count += 1;
                    let elapsed = rate_window_start.elapsed();
                    if elapsed >= std::time::Duration::from_secs(1) {
                        debug!(
                            "GameCube adapter snapshot publish rate: {:.1} Hz",
                            rate_window_count as f64 / elapsed.as_secs_f64()
                        );
                        rate_window_count = 0;
                        rate_window_start = std::time::Instant::now();
                    }
                }
            }
        })
        .expect("failed to spawn gc-adapter poll thread");

    Ok(GcAdapterSnapshot { ports, shutdown })
}
