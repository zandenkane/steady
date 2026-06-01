/// macOS backend stub.
///
/// A working implementation would use CGEventTapCreate from the
/// CoreGraphics framework to intercept pointer events at the
/// session/system level.
///
/// Requirements for a working implementation:
/// - App must have Accessibility permissions (System Settings > Privacy).
/// - Create a CGEventTap for kCGEventMouseMoved.
/// - The tap callback receives each event and can modify or discard it.
/// - The event tap must be added to the current thread's CFRunLoop.
/// - The main thread must run CFRunLoopRun() (event taps require
///   the main thread's run loop on macOS).
///
/// This stub compiles and implements the InputBackend trait but returns
/// an error from `run()`.
use crate::backend::types::{BackendAction, InputBackend, RawPointerEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MacOSBackend {
    stop_flag: Arc<AtomicBool>,
}

impl Default for MacOSBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOSBackend {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl InputBackend for MacOSBackend {
    fn run<F>(&mut self, _callback: F) -> Result<(), String>
    where
        F: FnMut(RawPointerEvent) -> Vec<BackendAction>,
    {
        Err("macOS CGEventTap backend not yet implemented. \
             The approach: CGEventTapCreate for kCGEventMouseMoved, \
             add tap to CFRunLoop, filter events in the tap callback, \
             modify CGEvent coordinates for filtered output."
            .to_string())
    }

    fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}
