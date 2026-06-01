//! Shared types for the platform backend layer.

/// A raw pointer event from the OS.
#[derive(Debug, Clone, Copy)]
pub struct RawPointerEvent {
    pub x: f64,
    pub y: f64,
    /// Timestamp in seconds (monotonic).
    pub timestamp: f64,
    /// Whether this event was injected by steady itself.
    pub injected: bool,
}

/// Actions the backend can perform after the pipeline processes an event.
#[derive(Debug, Clone, Copy)]
pub enum BackendAction {
    /// Move the cursor to the given absolute position.
    MoveTo(f64, f64),
    /// Inject a left-button click at the given position.
    Click(f64, f64),
}

/// Trait that platform backends implement.
pub trait InputBackend {
    /// Start intercepting pointer events. This call blocks (runs the
    /// message pump / event loop). The callback receives each raw event
    /// and returns a list of actions for the backend to execute.
    fn run<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: FnMut(RawPointerEvent) -> Vec<BackendAction>;

    /// Request graceful shutdown of the event loop.
    fn request_stop(&self);
}
