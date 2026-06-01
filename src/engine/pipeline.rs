/// Filter pipeline that chains tremor rejection, smoothing, click zone
/// snapping, and dwell detection in the correct order.
///
/// Pipeline order:
///   raw event -> tremor rejection -> smoothing -> click zone snap -> output
///   (dwell detection runs on the output of the pipeline)
use crate::config::Config;
use crate::engine::clickzone::{ClickZone, ClickZoneSnap};
use crate::engine::dwell::{DwellDetector, DwellEvent};
use crate::engine::smoothing::Smoother;
use crate::engine::stats::PipelineStats;
use crate::engine::tremor::TremorFilter;

/// What the pipeline decided to do with an input event.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterResult {
    /// Event was suppressed (tremor rejection ate it).
    Suppressed,
    /// Event was modified to new coordinates.
    Modified(f64, f64),
    /// A dwell click should fire at these coordinates.
    DwellClick(f64, f64),
}

/// Full filter pipeline state.
pub struct FilterPipeline {
    tremor: Option<TremorFilter>,
    smoother: Option<Smoother>,
    click_snap: Option<ClickZoneSnap>,
    dwell: Option<DwellDetector>,
    stats: PipelineStats,
}

impl FilterPipeline {
    pub fn from_config(config: &Config) -> Self {
        let tremor = if config.tremor.enabled {
            Some(TremorFilter::new(config.tremor.threshold))
        } else {
            None
        };

        let smoother = if config.smoothing.enabled {
            Some(Smoother::new(
                config.smoothing.min_cutoff,
                config.smoothing.beta,
            ))
        } else {
            None
        };

        let click_snap = if config.clickzones.enabled {
            let zones: Vec<ClickZone> = config
                .clickzones
                .zones
                .iter()
                .map(|z| ClickZone {
                    x: z.x,
                    y: z.y,
                    radius: z.radius,
                })
                .collect();
            Some(ClickZoneSnap::new(zones))
        } else {
            None
        };

        let dwell = if config.dwell.enabled {
            Some(DwellDetector::new(
                config.dwell.radius,
                config.dwell.time_ms,
            ))
        } else {
            None
        };

        Self {
            tremor,
            smoother,
            click_snap,
            dwell,
            stats: PipelineStats::new(),
        }
    }

    /// Process a raw pointer event. `timestamp` is in seconds.
    /// Returns a list of results (usually one Modified, possibly followed
    /// by a DwellClick).
    pub fn process(&mut self, x: f64, y: f64, timestamp: f64) -> Vec<FilterResult> {
        let mut results = Vec::new();
        self.stats.record_received();

        // Stage 1: Tremor rejection
        let (cx, cy) = if let Some(ref mut tremor) = self.tremor {
            match tremor.filter(x, y) {
                Some(pos) => {
                    self.stats.record_tremor_passed();
                    pos
                }
                None => {
                    self.stats.record_tremor_suppressed();
                    results.push(FilterResult::Suppressed);
                    return results;
                }
            }
        } else {
            (x, y)
        };

        // Stage 2: Smoothing
        let (sx, sy) = if let Some(ref mut smoother) = self.smoother {
            smoother.filter(cx, cy, timestamp)
        } else {
            (cx, cy)
        };

        // Stage 3: Click zone snap
        let (fx, fy) = if let Some(ref snap) = self.click_snap {
            let snapped = snap.snap(sx, sy);
            if (snapped.0 - sx).abs() > f64::EPSILON || (snapped.1 - sy).abs() > f64::EPSILON {
                self.stats.record_snap_activation();
            }
            snapped
        } else {
            (sx, sy)
        };

        results.push(FilterResult::Modified(fx, fy));

        // Stage 4: Dwell detection (runs on filtered output)
        if let Some(ref mut dwell) = self.dwell {
            let timestamp_ms = timestamp * 1000.0;
            if let DwellEvent::Click(dx, dy) = dwell.update(fx, fy, timestamp_ms) {
                self.stats.record_dwell_click();
                results.push(FilterResult::DwellClick(dx, dy));
            }
        }

        results
    }

    /// Get a snapshot of the current pipeline statistics.
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    pub fn reset(&mut self) {
        if let Some(ref mut t) = self.tremor {
            t.reset();
        }
        if let Some(ref mut s) = self.smoother {
            s.reset();
        }
        if let Some(ref mut d) = self.dwell {
            d.reset();
        }
        self.stats.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn pipeline_processes_basic_event() {
        let config = default_config();
        let mut pipe = FilterPipeline::from_config(&config);
        let results = pipe.process(100.0, 100.0, 0.0);
        assert!(!results.is_empty());
        match &results[0] {
            FilterResult::Modified(x, y) => {
                assert!((x - 100.0).abs() < 5.0);
                assert!((y - 100.0).abs() < 5.0);
            }
            _ => panic!("expected Modified result"),
        }
    }

    #[test]
    fn pipeline_suppresses_tremor() {
        let mut config = default_config();
        config.tremor.enabled = true;
        config.tremor.threshold = 10.0;
        config.smoothing.enabled = false;
        config.dwell.enabled = false;
        config.clickzones.enabled = false;

        let mut pipe = FilterPipeline::from_config(&config);
        // First event always accepted
        let r1 = pipe.process(100.0, 100.0, 0.0);
        assert!(matches!(r1[0], FilterResult::Modified(_, _)));

        // Small move should be suppressed
        let r2 = pipe.process(102.0, 101.0, 0.016);
        assert_eq!(r2[0], FilterResult::Suppressed);

        // Large move should pass
        let r3 = pipe.process(200.0, 200.0, 0.032);
        assert!(matches!(r3[0], FilterResult::Modified(_, _)));
    }

    #[test]
    fn pipeline_all_disabled_passes_through() {
        let mut config = default_config();
        config.tremor.enabled = false;
        config.smoothing.enabled = false;
        config.dwell.enabled = false;
        config.clickzones.enabled = false;

        let mut pipe = FilterPipeline::from_config(&config);
        let results = pipe.process(42.0, 84.0, 0.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], FilterResult::Modified(42.0, 84.0));
    }

    #[test]
    fn stats_track_events() {
        let mut config = default_config();
        config.tremor.enabled = true;
        config.tremor.threshold = 10.0;
        config.smoothing.enabled = false;
        config.dwell.enabled = false;
        config.clickzones.enabled = false;

        let mut pipe = FilterPipeline::from_config(&config);
        pipe.process(100.0, 100.0, 0.0);
        pipe.process(101.0, 100.0, 0.016); // suppressed
        pipe.process(200.0, 200.0, 0.032); // passed

        assert_eq!(pipe.stats().events_received, 3);
        assert_eq!(pipe.stats().tremor_passed, 2);
        assert_eq!(pipe.stats().tremor_suppressed, 1);
    }

    #[test]
    fn reset_clears_stats() {
        let config = default_config();
        let mut pipe = FilterPipeline::from_config(&config);
        pipe.process(100.0, 100.0, 0.0);
        pipe.reset();
        assert_eq!(pipe.stats().events_received, 0);
    }
}
