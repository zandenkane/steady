use steady::engine::tremor::TremorFilter;

#[test]
fn first_event_always_accepted() {
    let mut f = TremorFilter::new(5.0);
    assert!(f.filter(50.0, 50.0).is_some());
}

#[test]
fn micro_movement_rejected() {
    let mut f = TremorFilter::new(5.0);
    f.filter(100.0, 100.0);
    // Move 1px in each direction (distance ~1.41)
    assert_eq!(f.filter(101.0, 101.0), None);
}

#[test]
fn movement_at_threshold_accepted() {
    let mut f = TremorFilter::new(5.0);
    f.filter(0.0, 0.0);
    // Move exactly 5px along x axis
    assert!(f.filter(5.0, 0.0).is_some());
}

#[test]
fn diagonal_distance_calculated_correctly() {
    let mut f = TremorFilter::new(5.0);
    f.filter(0.0, 0.0);
    // Move (3, 4) = distance 5
    assert!(f.filter(3.0, 4.0).is_some());
    // Reset and test sub threshold diagonal
    f.reset();
    f.filter(0.0, 0.0);
    // Move (2, 2) = distance ~2.83
    assert_eq!(f.filter(2.0, 2.0), None);
}

#[test]
fn reset_clears_state() {
    let mut f = TremorFilter::new(5.0);
    f.filter(100.0, 100.0);
    f.reset();
    // After reset, the next event should be accepted regardless of position
    assert!(f.filter(101.0, 101.0).is_some());
}

#[test]
fn accepted_position_becomes_new_anchor() {
    let mut f = TremorFilter::new(5.0);
    f.filter(0.0, 0.0);
    f.filter(10.0, 0.0); // accepted, new anchor at (10, 0)
                         // Small move from new anchor should be rejected
    assert_eq!(f.filter(11.0, 0.0), None);
    // Large move from new anchor should be accepted
    assert!(f.filter(20.0, 0.0).is_some());
}

#[test]
fn zero_threshold_accepts_everything() {
    let mut f = TremorFilter::new(0.0);
    f.filter(0.0, 0.0);
    assert!(f.filter(0.001, 0.001).is_some());
}

#[test]
fn large_threshold_rejects_moderate_moves() {
    let mut f = TremorFilter::new(100.0);
    f.filter(0.0, 0.0);
    // 50px move should still be rejected with 100px threshold
    assert_eq!(f.filter(50.0, 0.0), None);
    // 100px move should pass
    assert!(f.filter(100.0, 0.0).is_some());
}

#[test]
fn negative_coordinates_work() {
    let mut f = TremorFilter::new(5.0);
    f.filter(-100.0, -200.0);
    assert_eq!(f.filter(-101.0, -201.0), None);
    assert!(f.filter(-200.0, -300.0).is_some());
}

#[test]
fn repeated_same_position_is_rejected() {
    let mut f = TremorFilter::new(5.0);
    f.filter(50.0, 50.0);
    // Same exact position should be rejected (distance 0)
    assert_eq!(f.filter(50.0, 50.0), None);
    assert_eq!(f.filter(50.0, 50.0), None);
}

#[test]
fn many_small_moves_all_rejected() {
    let mut f = TremorFilter::new(10.0);
    f.filter(100.0, 100.0);
    // Simulate hand tremor: many tiny random moves
    let offsets = [
        (1.0, 0.5),
        (-0.3, 1.2),
        (0.8, -0.6),
        (-1.0, 0.2),
        (0.5, 0.9),
    ];
    for &(dx, dy) in &offsets {
        assert_eq!(f.filter(100.0 + dx, 100.0 + dy), None);
    }
}

#[test]
fn sequential_large_moves_all_accepted() {
    let mut f = TremorFilter::new(5.0);
    let positions = [
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
        (0.0, 0.0),
    ];
    for &(x, y) in &positions {
        assert!(f.filter(x, y).is_some());
    }
}
