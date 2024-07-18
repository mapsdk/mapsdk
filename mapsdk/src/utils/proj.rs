use std::f64::consts::PI;

use crate::geo::Coord;

const WEB_MERCATOR_EARTH_RADIUS: f64 = 6378137.0;

pub fn lonlat_to_wm(coord: &Coord) -> Option<Coord> {
    let x = coord.x();
    let y = coord.y();

    if x >= -180.0 && x <= 180.0 && y >= -90.0 && y <= 90.0 {
        let x = (WEB_MERCATOR_EARTH_RADIUS * coord.x()).to_radians();
        let siny = (coord.y().to_radians()).sin();
        let y = WEB_MERCATOR_EARTH_RADIUS / 2.0 * ((1.0 + siny) / (1.0 - siny)).ln();

        if x.is_finite() && y.is_finite() {
            return Some(Coord::new(x, y));
        }
    }

    None
}

pub fn wm_to_lonlat(coord: &Coord) -> Option<Coord> {
    let x = coord.x();
    let y = coord.y();

    if x >= -20037508.34278924
        && x <= 20037508.34278924
        && y >= -20037508.34278924
        && y <= 20037508.34278924
    {
        let x_rad = coord.x() / WEB_MERCATOR_EARTH_RADIUS;
        let x = (x_rad - (x_rad / PI / 2.0).floor() * 2.0 * PI).to_degrees();
        let y =
            (PI / 2.0 - (-coord.y() / WEB_MERCATOR_EARTH_RADIUS).exp().atan() * 2.0).to_degrees();

        if x.is_finite() && y.is_finite() {
            return Some(Coord::new(x, y));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lonlat_to_wm() {
        assert_eq!(
            lonlat_to_wm(&Coord::new(0.0, 0.0)).unwrap(),
            Coord::new(0.0, 0.0)
        );
        assert_eq!(
            lonlat_to_wm(&Coord::new(180.0, 0.0)).unwrap(),
            Coord::new(20037508.34278924, 0.0)
        );
        assert_eq!(
            lonlat_to_wm(&Coord::new(180.0, 60.0)).unwrap(),
            Coord::new(20037508.34278924, 8399737.88981836)
        );
        assert!(lonlat_to_wm(&Coord::new(180.0, 90.0)).is_none());
        assert!(lonlat_to_wm(&Coord::new(-270.0, 0.0)).is_none());
    }

    #[test]
    fn test_wm_to_lonlat() {
        assert_eq!(
            wm_to_lonlat(&Coord::new(0.0, 0.0)).unwrap(),
            Coord::new(0.0, 0.0)
        );
        assert_eq!(
            wm_to_lonlat(&Coord::new(20037508.34278924, 0.0)).unwrap(),
            Coord::new(180.0, 0.0)
        );
        assert_eq!(
            wm_to_lonlat(&Coord::new(20037508.34278924, 20037508.34278924)).unwrap(),
            Coord::new(180.0, 85.05112877980659)
        );
        assert!(wm_to_lonlat(&Coord::new(20037509.0, 20037508.34278924)).is_none());
        assert!(wm_to_lonlat(&Coord::new(-20037508.34278924, -20037509.0)).is_none());
    }
}
