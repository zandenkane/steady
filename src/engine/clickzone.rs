/// Click zone snapping.
///
/// Given a list of defined zones (center point + radius), snaps the
/// pointer to the zone center when the pointer enters a zone. Zones
/// are defined in configuration. If the pointer is within multiple
/// zones, the nearest center wins.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ClickZone {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

#[derive(Debug, Clone)]
pub struct ClickZoneSnap {
    zones: Vec<ClickZone>,
}

impl ClickZoneSnap {
    pub fn new(zones: Vec<ClickZone>) -> Self {
        Self { zones }
    }

    /// Given a pointer position, return the snapped position.
    /// If the pointer is inside a zone, it snaps to that zone's center.
    /// If inside multiple zones, snaps to the nearest center.
    /// If outside all zones, returns the original position.
    pub fn snap(&self, x: f64, y: f64) -> (f64, f64) {
        let mut best: Option<(f64, &ClickZone)> = None;

        for zone in &self.zones {
            let dx = x - zone.x;
            let dy = y - zone.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= zone.radius {
                match best {
                    None => best = Some((dist, zone)),
                    Some((bd, _)) if dist < bd => best = Some((dist, zone)),
                    _ => {}
                }
            }
        }

        match best {
            Some((_, zone)) => (zone.x, zone.y),
            None => (x, y),
        }
    }

    pub fn set_zones(&mut self, zones: Vec<ClickZone>) {
        self.zones = zones;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_all_zones_passes_through() {
        let snap = ClickZoneSnap::new(vec![ClickZone {
            x: 100.0,
            y: 100.0,
            radius: 20.0,
        }]);
        assert_eq!(snap.snap(200.0, 200.0), (200.0, 200.0));
    }

    #[test]
    fn inside_zone_snaps_to_center() {
        let snap = ClickZoneSnap::new(vec![ClickZone {
            x: 100.0,
            y: 100.0,
            radius: 20.0,
        }]);
        assert_eq!(snap.snap(105.0, 95.0), (100.0, 100.0));
    }

    #[test]
    fn nearest_zone_wins() {
        let snap = ClickZoneSnap::new(vec![
            ClickZone {
                x: 100.0,
                y: 100.0,
                radius: 30.0,
            },
            ClickZone {
                x: 120.0,
                y: 100.0,
                radius: 30.0,
            },
        ]);
        // Point at (115, 100) is inside both zones. Closer to (120, 100).
        assert_eq!(snap.snap(115.0, 100.0), (120.0, 100.0));
    }

    #[test]
    fn empty_zones_passes_through() {
        let snap = ClickZoneSnap::new(vec![]);
        assert_eq!(snap.snap(50.0, 50.0), (50.0, 50.0));
    }
}
