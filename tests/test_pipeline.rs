use steady::config::{Config, ZoneEntry};
use steady::engine::pipeline::{FilterPipeline, FilterResult};

#[test]
fn all_filters_enabled_produces_modified() {
    let config = Config::default();
    let mut pipe = FilterPipeline::from_config(&config);
    let results = pipe.process(100.0, 100.0, 0.0);
    assert!(!results.is_empty());
    assert!(matches!(results[0], FilterResult::Modified(_, _)));
}

#[test]
fn all_filters_disabled_is_passthrough() {
    let mut config = Config::default();
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
fn tremor_only_suppresses_small_moves() {
    let mut config = Config::default();
    config.tremor.enabled = true;
    config.tremor.threshold = 10.0;
    config.smoothing.enabled = false;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    // First event accepted
    let r1 = pipe.process(100.0, 100.0, 0.0);
    assert!(matches!(r1[0], FilterResult::Modified(_, _)));

    // Small move suppressed
    let r2 = pipe.process(103.0, 102.0, 0.016);
    assert_eq!(r2[0], FilterResult::Suppressed);

    // Large move accepted
    let r3 = pipe.process(200.0, 200.0, 0.032);
    assert!(matches!(r3[0], FilterResult::Modified(_, _)));
    if let FilterResult::Modified(x, y) = r3[0] {
        assert!((x - 200.0).abs() < 0.01);
        assert!((y - 200.0).abs() < 0.01);
    }
}

#[test]
fn smoothing_only_attenuates_jitter() {
    let mut config = Config::default();
    config.tremor.enabled = false;
    config.smoothing.enabled = true;
    config.smoothing.min_cutoff = 0.3;
    config.smoothing.beta = 0.0;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);

    // Feed jittery input
    let jitter = [
        (100.0, 100.0),
        (110.0, 90.0),
        (90.0, 110.0),
        (105.0, 95.0),
        (95.0, 105.0),
    ];

    let mut last_x = 0.0;
    let mut last_y = 0.0;
    for (i, &(x, y)) in jitter.iter().enumerate() {
        let r = pipe.process(x, y, i as f64 * 0.016);
        if let FilterResult::Modified(rx, ry) = r[0] {
            last_x = rx;
            last_y = ry;
        }
    }

    // Smoothed output should be closer to center than the raw extremes
    assert!((last_x - 100.0).abs() < 15.0);
    assert!((last_y - 100.0).abs() < 15.0);
}

#[test]
fn dwell_fires_after_holding_still() {
    let mut config = Config::default();
    config.tremor.enabled = false;
    config.smoothing.enabled = false;
    config.dwell.enabled = true;
    config.dwell.time_ms = 500.0;
    config.dwell.radius = 10.0;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    // First event
    pipe.process(100.0, 100.0, 0.0);
    // Stay still for 500ms (timestamp in seconds)
    let results = pipe.process(101.0, 100.0, 0.5);
    assert!(results.len() >= 2);
    assert!(matches!(results[1], FilterResult::DwellClick(_, _)));
}

#[test]
fn pipeline_ordering_is_tremor_then_smooth_then_snap() {
    // When tremor rejects an event, smoothing should never see it
    let mut config = Config::default();
    config.tremor.enabled = true;
    config.tremor.threshold = 50.0; // Very aggressive
    config.smoothing.enabled = true;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    pipe.process(0.0, 0.0, 0.0);

    // Small move should be suppressed by tremor before reaching smoother
    let r = pipe.process(5.0, 5.0, 0.016);
    assert_eq!(r[0], FilterResult::Suppressed);
}

#[test]
fn reset_clears_all_filter_state() {
    let config = Config::default();
    let mut pipe = FilterPipeline::from_config(&config);
    pipe.process(100.0, 100.0, 0.0);
    pipe.process(200.0, 200.0, 0.016);
    pipe.reset();
    // After reset, first event should pass through cleanly
    let r = pipe.process(500.0, 500.0, 1.0);
    assert!(matches!(r[0], FilterResult::Modified(_, _)));
}

#[test]
fn clickzone_snap_in_pipeline() {
    let mut config = Config::default();
    config.tremor.enabled = false;
    config.smoothing.enabled = false;
    config.dwell.enabled = false;
    config.clickzones.enabled = true;
    config.clickzones.zones = vec![ZoneEntry {
        x: 500.0,
        y: 500.0,
        radius: 30.0,
    }];

    let mut pipe = FilterPipeline::from_config(&config);
    // Position inside the zone
    let results = pipe.process(510.0, 505.0, 0.0);
    assert_eq!(results[0], FilterResult::Modified(500.0, 500.0));

    // Position outside all zones
    let results2 = pipe.process(100.0, 100.0, 0.016);
    assert_eq!(results2[0], FilterResult::Modified(100.0, 100.0));
}

#[test]
fn combined_tremor_and_smoothing() {
    let mut config = Config::default();
    config.tremor.enabled = true;
    config.tremor.threshold = 3.0;
    config.smoothing.enabled = true;
    config.smoothing.min_cutoff = 1.0;
    config.smoothing.beta = 0.007;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    // First event always passes
    let r = pipe.process(100.0, 100.0, 0.0);
    assert!(matches!(r[0], FilterResult::Modified(_, _)));

    // Large move passes through tremor and gets smoothed
    let r2 = pipe.process(200.0, 200.0, 0.016);
    if let FilterResult::Modified(x, y) = r2[0] {
        // Should be somewhere between 100 and 200 due to smoothing
        assert!(x > 100.0 && x <= 200.0, "x={}", x);
        assert!(y > 100.0 && y <= 200.0, "y={}", y);
    } else {
        panic!("expected Modified result");
    }
}

#[test]
fn stats_track_tremor_suppression() {
    let mut config = Config::default();
    config.tremor.enabled = true;
    config.tremor.threshold = 20.0;
    config.smoothing.enabled = false;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    pipe.process(100.0, 100.0, 0.0); // accepted
    pipe.process(101.0, 100.0, 0.016); // suppressed
    pipe.process(102.0, 101.0, 0.032); // suppressed
    pipe.process(200.0, 200.0, 0.048); // accepted

    let stats = pipe.stats();
    assert_eq!(stats.events_received, 4);
    assert_eq!(stats.tremor_passed, 2);
    assert_eq!(stats.tremor_suppressed, 2);
}

#[test]
fn stats_track_dwell_clicks() {
    let mut config = Config::default();
    config.tremor.enabled = false;
    config.smoothing.enabled = false;
    config.dwell.enabled = true;
    config.dwell.time_ms = 100.0;
    config.dwell.radius = 10.0;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);
    pipe.process(100.0, 100.0, 0.0);
    pipe.process(101.0, 100.0, 0.1); // triggers dwell click

    assert_eq!(pipe.stats().dwell_clicks, 1);
}

#[test]
fn many_events_in_sequence() {
    let mut config = Config::default();
    config.tremor.enabled = false;
    config.smoothing.enabled = true;
    config.smoothing.min_cutoff = 1.0;
    config.smoothing.beta = 0.5;
    config.dwell.enabled = false;
    config.clickzones.enabled = false;

    let mut pipe = FilterPipeline::from_config(&config);

    // Simulate 100 events along a diagonal
    for i in 0..100 {
        let t = i as f64 * 0.016;
        let x = i as f64 * 5.0;
        let y = i as f64 * 3.0;
        let results = pipe.process(x, y, t);
        assert!(!results.is_empty());
        if let FilterResult::Modified(rx, ry) = results[0] {
            assert!(rx.is_finite());
            assert!(ry.is_finite());
        }
    }

    assert_eq!(pipe.stats().events_received, 100);
}
