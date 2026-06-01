/// Tremor rejection filter.
///
/// Discards pointer moves smaller than a configurable pixel radius
/// from the last accepted position. Micro-movements caused by hand
/// tremor get eaten; intentional moves pass through.

#[derive(Debug, Clone)]
pub struct TremorFilter {
    threshold: f64,
    last_accepted: Option<(f64, f64)>,
}

impl TremorFilter {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            last_accepted: None,
        }
    }

    /// Feed a new position. Returns `Some((x, y))` if the movement
    /// exceeds the threshold (accepted), or `None` if suppressed.
    pub fn filter(&mut self, x: f64, y: f64) -> Option<(f64, f64)> {
        match self.last_accepted {
            None => {
                self.last_accepted = Some((x, y));
                Some((x, y))
            }
            Some((lx, ly)) => {
                let dx = x - lx;
                let dy = y - ly;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist >= self.threshold {
                    self.last_accepted = Some((x, y));
                    Some((x, y))
                } else {
                    None
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.last_accepted = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_event_always_passes() {
        let mut f = TremorFilter::new(3.0);
        assert_eq!(f.filter(10.0, 10.0), Some((10.0, 10.0)));
    }

    #[test]
    fn small_movement_suppressed() {
        let mut f = TremorFilter::new(5.0);
        f.filter(10.0, 10.0);
        assert_eq!(f.filter(12.0, 11.0), None);
    }

    #[test]
    fn large_movement_passes() {
        let mut f = TremorFilter::new(5.0);
        f.filter(10.0, 10.0);
        assert_eq!(f.filter(20.0, 10.0), Some((20.0, 10.0)));
    }
}
