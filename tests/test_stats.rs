use steady::engine::stats::PipelineStats;

#[test]
fn fresh_stats_have_zero_rejection_rate() {
    let s = PipelineStats::new();
    assert!((s.tremor_rejection_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn rejection_rate_with_mixed_events() {
    let mut s = PipelineStats::new();
    for _ in 0..7 {
        s.record_tremor_passed();
    }
    for _ in 0..3 {
        s.record_tremor_suppressed();
    }
    assert!((s.tremor_rejection_rate() - 0.3).abs() < 0.001);
}

#[test]
fn reset_brings_everything_to_zero() {
    let mut s = PipelineStats::new();
    s.record_received();
    s.record_received();
    s.record_tremor_passed();
    s.record_tremor_suppressed();
    s.record_dwell_click();
    s.record_snap_activation();
    s.reset();
    assert_eq!(s.events_received, 0);
    assert_eq!(s.tremor_passed, 0);
    assert_eq!(s.tremor_suppressed, 0);
    assert_eq!(s.dwell_clicks, 0);
    assert_eq!(s.snap_activations, 0);
}

#[test]
fn high_volume_counting() {
    let mut s = PipelineStats::new();
    for _ in 0..100_000 {
        s.record_received();
        s.record_tremor_passed();
    }
    assert_eq!(s.events_received, 100_000);
    assert_eq!(s.tremor_passed, 100_000);
}
