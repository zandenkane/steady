/// One Euro Filter for speed-adaptive pointer smoothing.
///
/// Based on Casiez et al. (CHI 2012). Applies heavy smoothing when the
/// pointer is near-still (kills tremor jitter) and light smoothing when
/// moving fast (low latency for intentional gestures).
///
/// Two parameters control behavior:
/// - `min_cutoff`: minimum cutoff frequency (Hz). Lower = more smoothing at rest.
/// - `beta`: speed coefficient. Higher = less smoothing during fast moves.
use std::f64::consts::PI;

#[derive(Debug, Clone)]
struct LowPassFilter {
    y_prev: Option<f64>,
    alpha: f64,
}

impl LowPassFilter {
    fn new() -> Self {
        Self {
            y_prev: None,
            alpha: 1.0,
        }
    }

    fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    fn filter(&mut self, value: f64) -> f64 {
        match self.y_prev {
            None => {
                self.y_prev = Some(value);
                value
            }
            Some(prev) => {
                let out = self.alpha * value + (1.0 - self.alpha) * prev;
                self.y_prev = Some(out);
                out
            }
        }
    }

    fn last(&self) -> Option<f64> {
        self.y_prev
    }

    fn reset(&mut self) {
        self.y_prev = None;
    }
}

fn smoothing_factor(te: f64, cutoff: f64) -> f64 {
    let r = 2.0 * PI * cutoff * te;
    r / (r + 1.0)
}

#[derive(Debug, Clone)]
pub struct OneEuroFilter {
    min_cutoff: f64,
    beta: f64,
    d_cutoff: f64,
    x_filter: LowPassFilter,
    dx_filter: LowPassFilter,
    last_time: Option<f64>,
}

impl OneEuroFilter {
    pub fn new(min_cutoff: f64, beta: f64) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff: 1.0,
            x_filter: LowPassFilter::new(),
            dx_filter: LowPassFilter::new(),
            last_time: None,
        }
    }

    /// Filter a single value at the given timestamp (seconds).
    /// Returns the smoothed value.
    pub fn filter(&mut self, value: f64, timestamp: f64) -> f64 {
        let te = match self.last_time {
            Some(lt) => {
                let dt = timestamp - lt;
                if dt <= 0.0 {
                    1.0 / 120.0
                } else {
                    dt
                }
            }
            None => 1.0 / 120.0,
        };
        self.last_time = Some(timestamp);

        // Estimate derivative (speed)
        let dx = match self.x_filter.last() {
            Some(prev) => (value - prev) / te,
            None => 0.0,
        };

        let edx = {
            self.dx_filter
                .set_alpha(smoothing_factor(te, self.d_cutoff));
            self.dx_filter.filter(dx)
        };

        // Adaptive cutoff based on speed
        let cutoff = self.min_cutoff + self.beta * edx.abs();

        self.x_filter.set_alpha(smoothing_factor(te, cutoff));
        self.x_filter.filter(value)
    }

    pub fn reset(&mut self) {
        self.x_filter.reset();
        self.dx_filter.reset();
        self.last_time = None;
    }
}

/// Paired One Euro filters for 2D pointer smoothing.
#[derive(Debug, Clone)]
pub struct Smoother {
    x_filter: OneEuroFilter,
    y_filter: OneEuroFilter,
}

impl Smoother {
    pub fn new(min_cutoff: f64, beta: f64) -> Self {
        Self {
            x_filter: OneEuroFilter::new(min_cutoff, beta),
            y_filter: OneEuroFilter::new(min_cutoff, beta),
        }
    }

    /// Smooth a 2D position. `timestamp` is in seconds.
    pub fn filter(&mut self, x: f64, y: f64, timestamp: f64) -> (f64, f64) {
        let sx = self.x_filter.filter(x, timestamp);
        let sy = self.y_filter.filter(y, timestamp);
        (sx, sy)
    }

    pub fn reset(&mut self) {
        self.x_filter.reset();
        self.y_filter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sample_passes_through() {
        let mut s = Smoother::new(1.0, 0.0);
        let (x, y) = s.filter(100.0, 200.0, 0.0);
        assert!((x - 100.0).abs() < 0.01);
        assert!((y - 200.0).abs() < 0.01);
    }

    #[test]
    fn smoothing_reduces_jitter() {
        let mut s = Smoother::new(0.5, 0.0);
        // Simulate jittery input around (100, 100)
        let jitter = [
            (102.0, 98.0),
            (97.0, 103.0),
            (101.0, 99.0),
            (98.0, 102.0),
            (100.0, 100.0),
        ];
        let mut last = (0.0, 0.0);
        for (i, &(x, y)) in jitter.iter().enumerate() {
            last = s.filter(x, y, i as f64 * 0.016);
        }
        // After several jittery samples, output should be closer to center
        // than the raw jitter extremes
        assert!((last.0 - 100.0).abs() < 5.0);
        assert!((last.1 - 100.0).abs() < 5.0);
    }

    #[test]
    fn fast_movement_has_low_lag() {
        let mut s = Smoother::new(1.0, 0.5);
        s.filter(0.0, 0.0, 0.0);
        // Big jump at high speed
        let (x, _) = s.filter(500.0, 0.0, 0.016);
        // With beta > 0, fast movement should track closely
        // (not lag far behind the target)
        assert!(x > 100.0, "fast movement should be tracked, got {}", x);
    }
}
