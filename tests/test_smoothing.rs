use steady::engine::smoothing::Smoother;

#[test]
fn first_sample_is_identity() {
    let mut s = Smoother::new(1.0, 0.0);
    let (x, y) = s.filter(200.0, 300.0, 0.0);
    assert!((x - 200.0).abs() < 0.001);
    assert!((y - 300.0).abs() < 0.001);
}

#[test]
fn jitter_gets_smoothed() {
    let mut s = Smoother::new(0.3, 0.0); // Aggressive smoothing
    let jitter: Vec<(f64, f64)> = vec![
        (100.0, 100.0),
        (105.0, 95.0),
        (95.0, 105.0),
        (103.0, 97.0),
        (97.0, 103.0),
        (101.0, 99.0),
        (99.0, 101.0),
        (100.0, 100.0),
    ];

    let mut last = (0.0, 0.0);
    for (i, &(x, y)) in jitter.iter().enumerate() {
        last = s.filter(x, y, i as f64 * 0.016);
    }

    // Smoothed output should be near center, not at extremes
    assert!(
        (last.0 - 100.0).abs() < 10.0,
        "x should be near 100, got {}",
        last.0
    );
    assert!(
        (last.1 - 100.0).abs() < 10.0,
        "y should be near 100, got {}",
        last.1
    );
}

#[test]
fn fast_movement_tracked_with_high_beta() {
    let mut s = Smoother::new(1.0, 1.0); // High beta = fast tracking
    s.filter(0.0, 0.0, 0.0);
    let (x, _) = s.filter(1000.0, 0.0, 0.016);
    // With high beta and fast movement, should track well beyond halfway
    assert!(x > 200.0, "fast move should track closely, got {}", x);
}

#[test]
fn slow_movement_gets_more_smoothing() {
    let mut slow = Smoother::new(0.5, 0.5);
    let mut fast = Smoother::new(0.5, 0.5);

    // Slow path: small steps
    slow.filter(0.0, 0.0, 0.0);
    let (slow_x, _) = slow.filter(2.0, 0.0, 0.016);

    // Fast path: big jump
    fast.filter(0.0, 0.0, 0.0);
    let (fast_x, _) = fast.filter(200.0, 0.0, 0.016);

    // Slow movement should be more damped (proportionally closer to origin)
    let slow_ratio = slow_x / 2.0;
    let fast_ratio = fast_x / 200.0;
    assert!(
        fast_ratio > slow_ratio,
        "fast movement should track proportionally better: fast_ratio={}, slow_ratio={}",
        fast_ratio,
        slow_ratio
    );
}

#[test]
fn reset_allows_fresh_start() {
    let mut s = Smoother::new(1.0, 0.0);
    s.filter(100.0, 100.0, 0.0);
    s.filter(200.0, 200.0, 0.016);
    s.reset();
    // After reset, next sample should pass through cleanly
    let (x, y) = s.filter(500.0, 500.0, 1.0);
    assert!((x - 500.0).abs() < 0.001);
    assert!((y - 500.0).abs() < 0.001);
}

#[test]
fn constant_input_converges() {
    let mut s = Smoother::new(1.0, 0.0);
    // Feed the same point 20 times at 60Hz
    let mut last = (0.0, 0.0);
    for i in 0..20 {
        last = s.filter(300.0, 400.0, i as f64 * 0.016);
    }
    // Should converge very close to the input value
    assert!(
        (last.0 - 300.0).abs() < 1.0,
        "x should converge to 300, got {}",
        last.0
    );
    assert!(
        (last.1 - 400.0).abs() < 1.0,
        "y should converge to 400, got {}",
        last.1
    );
}

#[test]
fn zero_beta_gives_consistent_smoothing() {
    // With beta=0, smoothing amount should not depend on speed
    let mut s = Smoother::new(1.0, 0.0);
    s.filter(0.0, 0.0, 0.0);
    let (x1, _) = s.filter(10.0, 0.0, 0.016);
    let ratio1 = x1 / 10.0;

    let mut s2 = Smoother::new(1.0, 0.0);
    s2.filter(0.0, 0.0, 0.0);
    let (x2, _) = s2.filter(100.0, 0.0, 0.016);
    let ratio2 = x2 / 100.0;

    // Both ratios should be similar since beta=0 means no speed adaptation
    assert!(
        (ratio1 - ratio2).abs() < 0.1,
        "zero beta should give consistent smoothing: ratio1={}, ratio2={}",
        ratio1,
        ratio2
    );
}

#[test]
fn negative_coordinates_smooth_correctly() {
    let mut s = Smoother::new(1.0, 0.0);
    let (x, y) = s.filter(-100.0, -200.0, 0.0);
    assert!((x - (-100.0)).abs() < 0.001);
    assert!((y - (-200.0)).abs() < 0.001);
}

#[test]
fn large_time_gap_does_not_panic() {
    let mut s = Smoother::new(1.0, 0.5);
    s.filter(0.0, 0.0, 0.0);
    // 10 second gap
    let (x, y) = s.filter(100.0, 100.0, 10.0);
    // Should still produce finite output
    assert!(x.is_finite());
    assert!(y.is_finite());
}
