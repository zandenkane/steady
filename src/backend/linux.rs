/// Linux backend stub.
///
/// A working implementation would use evdev to grab the pointer device
/// (exclusive access via EVIOCGRAB ioctl), read raw events, pass them
/// through the filter pipeline, and reinject filtered events through
/// a uinput virtual device.
///
/// Requirements for a working implementation:
/// - User must be in the `input` group or run as root.
/// - Identify the correct /dev/input/eventN device for the pointer.
/// - Create a uinput virtual device with matching capabilities.
/// - Grab the real device to prevent duplicate events.
/// - Read EV_REL / EV_ABS events, filter, write to uinput.
///
/// This stub compiles and implements the InputBackend trait but returns
/// an error from `run()`.
use crate::backend::types::{BackendAction, InputBackend, RawPointerEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct LinuxBackend {
    stop_flag: Arc<AtomicBool>,
}

impl Default for LinuxBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxBackend {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl InputBackend for LinuxBackend {
    fn run<F>(&mut self, _callback: F) -> Result<(), String>
    where
        F: FnMut(RawPointerEvent) -> Vec<BackendAction>,
    {
        Err("Linux evdev+uinput backend not yet implemented. \
             The approach: grab /dev/input/eventN via EVIOCGRAB, \
             create a uinput virtual device, read raw EV_REL/EV_ABS events, \
             filter through the pipeline, reinject via uinput write."
            .to_string())
    }

    fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}
