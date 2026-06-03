use steady::engine::clickzone::{ClickZone, ClickZoneSnap};

#[test]
fn no_zones_is_passthrough() {
    let snap = ClickZoneSnap::new(vec![]);
    assert_eq!(snap.snap(42.0, 84.0), (42.0, 84.0));
}

#[test]
fn outside_zone_passes_through() {
    let snap = ClickZoneSnap::new(vec![ClickZone {
        x: 500.0,
        y: 500.0,
        radius: 20.0,
    }]);
    assert_eq!(snap.snap(100.0, 100.0), (100.0, 100.0));
}

#[test]
fn inside_zone_snaps_to_center() {
    let snap = ClickZoneSnap::new(vec![ClickZone {
        x: 500.0,
        y: 500.0,
        radius: 30.0,
    }]);
    // 10px from center, well within the 30px radius
    assert_eq!(snap.snap(510.0, 505.0), (500.0, 500.0));
}

#[test]
fn on_edge_snaps() {
    let snap = ClickZoneSnap::new(vec![ClickZone {
        x: 100.0,
        y: 100.0,
        radius: 20.0,
    }]);
    // Exactly on the edge (20px away along x)
    assert_eq!(snap.snap(120.0, 100.0), (100.0, 100.0));
}

#[test]
fn just_outside_does_not_snap() {
    let snap = ClickZoneSnap::new(vec![ClickZone {
        x: 100.0,
        y: 100.0,
        radius: 20.0,
    }]);
    // 20.1px away
    let result = snap.snap(120.1, 100.0);
    assert_eq!(result, (120.1, 100.0));
}

#[test]
fn overlapping_zones_picks_nearest() {
    let snap = ClickZoneSnap::new(vec![
        ClickZone {
            x: 100.0,
            y: 100.0,
            radius: 40.0,
        },
        ClickZone {
            x: 130.0,
            y: 100.0,
            radius: 40.0,
        },
    ]);
    // Point at (125, 100): inside both, closer to (130, 100) at dist 5 vs (100,100) at dist 25
    assert_eq!(snap.snap(125.0, 100.0), (130.0, 100.0));
}

#[test]
fn multiple_zones_only_matching_one() {
    let snap = ClickZoneSnap::new(vec![
        ClickZone {
            x: 100.0,
            y: 100.0,
            radius: 20.0,
        },
        ClickZone {
            x: 500.0,
            y: 500.0,
            radius: 20.0,
        },
    ]);
    assert_eq!(snap.snap(105.0, 100.0), (100.0, 100.0));
    assert_eq!(snap.snap(505.0, 500.0), (500.0, 500.0));
    assert_eq!(snap.snap(300.0, 300.0), (300.0, 300.0));
}

#[test]
fn set_zones_updates_behavior() {
    let mut snap = ClickZoneSnap::new(vec![]);
    assert_eq!(snap.snap(100.0, 100.0), (100.0, 100.0));

    snap.set_zones(vec![ClickZone {
        x: 100.0,
        y: 100.0,
        radius: 20.0,
    }]);
    assert_eq!(snap.snap(105.0, 100.0), (100.0, 100.0));
}
