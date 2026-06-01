/// Dwell-click detector.
///
/// Watches the filtered pointer position. If the position stays within
/// `radius` pixels of the dwell anchor for `time_ms` milliseconds,
/// fires a click event. Runs on the filtered stream so that
/// micro-tremor does not constantly reset the timer.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DwellEvent {
    /// Pointer is dwelling but timer has not elapsed yet.
    Dwelling,
    /// Dwell timer elapsed, fire a click at (x, y).
    Click(f64, f64),
    /// Pointer moved outside dwell radius, timer reset.
    Moved,
}

#[derive(Debug, Clone)]
pub struct DwellDetector {
    radius: f64,
    time_ms: f64,
    anchor: Option<(f64, f64)>,
    anchor_time: Option<f64>,
    fired: bool,
}

impl DwellDetector {
    pub fn new(radius: f64, time_ms: f64) -> Self {
        Self {
            radius,
            time_ms,
            anchor: None,
            anchor_time: None,
            fired: false,
        }
    }

    /// Feed a filtered position with timestamp in milliseconds.
    /// Returns the dwell state.
    pub fn update(&mut self, x: f64, y: f64, timestamp_ms: f64) -> DwellEvent {
        match (self.anchor, self.anchor_time) {
            (Some((ax, ay)), Some(at)) => {
                let dx = x - ax;
                let dy = y - ay;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > self.radius {
                    // Moved outside dwell zone, reset
                    self.anchor = Some((x, y));
                    self.anchor_time = Some(timestamp_ms);
                    self.fired = false;
                    DwellEvent::Moved
                } else if !self.fired && (timestamp_ms - at) >= self.time_ms {
                    // Dwelled long enough, fire click
                    self.fired = true;
                    DwellEvent::Click(ax, ay)
                } else {
                    DwellEvent::Dwelling
                }
            }
            _ => {
                self.anchor = Some((x, y));
                self.anchor_time = Some(timestamp_ms);
                self.fired = false;
                DwellEvent::Dwelling
            }
        }
    }

    pub fn reset(&mut self) {
        self.anchor = None;
        self.anchor_time = None;
        self.fired = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_event_is_dwelling() {
        let mut d = DwellDetector::new(10.0, 800.0);
        assert_eq!(d.update(100.0, 100.0, 0.0), DwellEvent::Dwelling);
    }

    #[test]
    fn fires_click_after_dwell_time() {
        let mut d = DwellDetector::new(10.0, 800.0);
        d.update(100.0, 100.0, 0.0);
        // Stay within radius for 800ms
        assert_eq!(d.update(102.0, 101.0, 400.0), DwellEvent::Dwelling);
        assert_eq!(
            d.update(101.0, 99.0, 800.0),
            DwellEvent::Click(100.0, 100.0)
        );
    }

    #[test]
    fn movement_resets_timer() {
        let mut d = DwellDetector::new(10.0, 800.0);
        d.update(100.0, 100.0, 0.0);
        d.update(102.0, 101.0, 400.0);
        // Move far away
        assert_eq!(d.update(200.0, 200.0, 500.0), DwellEvent::Moved);
        // Timer should be reset, dwelling should not fire yet
        assert_eq!(d.update(201.0, 199.0, 1000.0), DwellEvent::Dwelling);
    }

    #[test]
    fn does_not_fire_twice() {
        let mut d = DwellDetector::new(10.0, 800.0);
        d.update(100.0, 100.0, 0.0);
        d.update(101.0, 100.0, 800.0); // fires
                                       // Subsequent updates in same zone should not re-fire
        assert_eq!(d.update(100.0, 101.0, 900.0), DwellEvent::Dwelling);
    }
}
