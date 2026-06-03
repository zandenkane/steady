use steady::engine::dwell::{DwellDetector, DwellEvent};

#[test]
fn first_update_is_dwelling() {
    let mut d = DwellDetector::new(10.0, 800.0);
    assert_eq!(d.update(50.0, 50.0, 0.0), DwellEvent::Dwelling);
}

#[test]
fn click_fires_after_dwell_time() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    assert_eq!(d.update(101.0, 99.0, 250.0), DwellEvent::Dwelling);
    assert_eq!(
        d.update(100.0, 101.0, 500.0),
        DwellEvent::Click(100.0, 100.0)
    );
}

#[test]
fn movement_outside_radius_resets() {
    let mut d = DwellDetector::new(10.0, 800.0);
    d.update(100.0, 100.0, 0.0);
    d.update(102.0, 101.0, 400.0);
    // Move well outside the 10px radius
    assert_eq!(d.update(200.0, 200.0, 500.0), DwellEvent::Moved);
}

#[test]
fn does_not_double_fire() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    // Fire
    let ev = d.update(101.0, 100.0, 500.0);
    assert_eq!(ev, DwellEvent::Click(100.0, 100.0));
    // Continue in same zone, should not fire again
    assert_eq!(d.update(100.0, 101.0, 600.0), DwellEvent::Dwelling);
    assert_eq!(d.update(101.0, 101.0, 700.0), DwellEvent::Dwelling);
}

#[test]
fn can_fire_again_after_leaving_and_returning() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    d.update(101.0, 100.0, 500.0); // fires

    // Leave the zone
    d.update(300.0, 300.0, 600.0);
    // Return and dwell again
    d.update(100.0, 100.0, 700.0);
    let ev = d.update(101.0, 100.0, 1200.0);
    assert_eq!(ev, DwellEvent::Click(100.0, 100.0));
}

#[test]
fn large_radius_is_more_forgiving() {
    let mut d = DwellDetector::new(50.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    // Move 30px away (within 50px radius)
    assert_eq!(d.update(130.0, 100.0, 250.0), DwellEvent::Dwelling);
    assert_eq!(
        d.update(125.0, 105.0, 500.0),
        DwellEvent::Click(100.0, 100.0)
    );
}

#[test]
fn reset_clears_dwell_state() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    d.update(101.0, 100.0, 400.0);
    d.reset();
    // After reset, should start fresh
    assert_eq!(d.update(100.0, 100.0, 500.0), DwellEvent::Dwelling);
}

#[test]
fn very_short_dwell_time_fires_quickly() {
    let mut d = DwellDetector::new(10.0, 50.0);
    d.update(100.0, 100.0, 0.0);
    assert_eq!(
        d.update(101.0, 100.0, 50.0),
        DwellEvent::Click(100.0, 100.0)
    );
}

#[test]
fn exactly_on_boundary_does_not_fire_prematurely() {
    let mut d = DwellDetector::new(10.0, 800.0);
    d.update(100.0, 100.0, 0.0);
    // Just before the timeout
    assert_eq!(d.update(101.0, 100.0, 799.0), DwellEvent::Dwelling);
    // Exactly at the timeout
    assert_eq!(
        d.update(101.0, 100.0, 800.0),
        DwellEvent::Click(100.0, 100.0)
    );
}

#[test]
fn click_fires_at_anchor_not_current_position() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(100.0, 100.0, 0.0);
    // Stay within radius but move around
    let ev = d.update(105.0, 103.0, 500.0);
    // Click should be at the anchor (100, 100), not at (105, 103)
    assert_eq!(ev, DwellEvent::Click(100.0, 100.0));
}

#[test]
fn rapid_zone_transitions() {
    let mut d = DwellDetector::new(10.0, 500.0);
    // Keep moving to new positions, never dwelling long enough
    d.update(100.0, 100.0, 0.0);
    assert_eq!(d.update(200.0, 200.0, 100.0), DwellEvent::Moved);
    assert_eq!(d.update(300.0, 300.0, 200.0), DwellEvent::Moved);
    assert_eq!(d.update(400.0, 400.0, 300.0), DwellEvent::Moved);
    // None of these should trigger a click
}

#[test]
fn negative_coordinates_work() {
    let mut d = DwellDetector::new(10.0, 500.0);
    d.update(-100.0, -200.0, 0.0);
    assert_eq!(
        d.update(-99.0, -201.0, 500.0),
        DwellEvent::Click(-100.0, -200.0)
    );
}
