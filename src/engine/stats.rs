/// Pipeline statistics tracker.
///
/// Counts how many events each filter stage handled, so the user can
/// see how much work each filter is doing. Useful for tuning thresholds.

#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    /// Total raw events received.
    pub events_received: u64,
    /// Events suppressed by the tremor filter.
    pub tremor_suppressed: u64,
    /// Events that passed through tremor (or tremor was disabled).
    pub tremor_passed: u64,
    /// Dwell clicks fired.
    pub dwell_clicks: u64,
    /// Events where click zone snap changed the position.
    pub snap_activations: u64,
}

impl PipelineStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_received(&mut self) {
        self.events_received += 1;
    }

    pub fn record_tremor_suppressed(&mut self) {
        self.tremor_suppressed += 1;
    }

    pub fn record_tremor_passed(&mut self) {
        self.tremor_passed += 1;
    }

    pub fn record_dwell_click(&mut self) {
        self.dwell_clicks += 1;
    }

    pub fn record_snap_activation(&mut self) {
        self.snap_activations += 1;
    }

    /// Fraction of events rejected by tremor (0.0 to 1.0).
    /// Returns 0.0 if no events have been processed.
    pub fn tremor_rejection_rate(&self) -> f64 {
        let total = self.tremor_suppressed + self.tremor_passed;
        if total == 0 {
            return 0.0;
        }
        self.tremor_suppressed as f64 / total as f64
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stats_are_zero() {
        let s = PipelineStats::new();
        assert_eq!(s.events_received, 0);
        assert_eq!(s.tremor_suppressed, 0);
        assert_eq!(s.tremor_passed, 0);
        assert_eq!(s.dwell_clicks, 0);
        assert_eq!(s.snap_activations, 0);
    }

    #[test]
    fn rejection_rate_no_events() {
        let s = PipelineStats::new();
        assert!((s.tremor_rejection_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rejection_rate_all_suppressed() {
        let mut s = PipelineStats::new();
        s.tremor_suppressed = 10;
        assert!((s.tremor_rejection_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rejection_rate_half() {
        let mut s = PipelineStats::new();
        s.tremor_suppressed = 5;
        s.tremor_passed = 5;
        assert!((s.tremor_rejection_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reset_clears_all() {
        let mut s = PipelineStats::new();
        s.events_received = 100;
        s.tremor_suppressed = 50;
        s.dwell_clicks = 3;
        s.reset();
        assert_eq!(s.events_received, 0);
        assert_eq!(s.tremor_suppressed, 0);
        assert_eq!(s.dwell_clicks, 0);
    }

    #[test]
    fn record_methods_increment() {
        let mut s = PipelineStats::new();
        s.record_received();
        s.record_received();
        s.record_tremor_passed();
        s.record_tremor_suppressed();
        s.record_dwell_click();
        s.record_snap_activation();
        assert_eq!(s.events_received, 2);
        assert_eq!(s.tremor_passed, 1);
        assert_eq!(s.tremor_suppressed, 1);
        assert_eq!(s.dwell_clicks, 1);
        assert_eq!(s.snap_activations, 1);
    }
}
