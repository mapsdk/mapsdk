use crate::geo::{Bbox, Coord};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TileId {
    pub z: usize,
    pub x: i32,
    pub y: i32,
}

impl ToString for TileId {
    fn to_string(&self) -> String {
        format!("{}/{}/{}", self.z, self.x, self.y)
    }
}

#[derive(Clone, Debug)]
pub struct Tiling {
    pub tile_size: u32,
    pub origin_x: f64,
    pub origin_y: f64,
    pub zoom_resolutions: Vec<f64>,
}

impl Default for Tiling {
    /// Use Google's tiling scheme as default
    fn default() -> Self {
        const TILE_SIZE: u32 = 256;
        const MAP_MAX: f64 = 20037508.34278924;
        const BASE_RES: f64 = MAP_MAX / TILE_SIZE as f64;

        Self {
            tile_size: TILE_SIZE,
            origin_x: -MAP_MAX,
            origin_y: MAP_MAX,
            zoom_resolutions: (0..24).map(|i| BASE_RES / (2.0_f64).powi(i - 1)).collect(),
        }
    }
}

impl Tiling {
    pub fn drill_down_tile_ids(&self, tile_id: &TileId, level: u32) -> Vec<TileId> {
        let mut tile_ids: Vec<TileId> = Vec::new();

        if tile_id.z < self.zoom_resolutions.len() - 1 {
            let child_z = tile_id.z + level as usize;
            let factor = 2_i32.pow(level);

            for i in 0..factor {
                for j in 0..factor {
                    tile_ids.push(TileId {
                        z: child_z,
                        x: tile_id.x * factor + i,
                        y: tile_id.y * factor + j,
                    });
                }
            }
        }

        tile_ids
    }

    pub fn get_closest_lower_zoom(&self, resolution: f64) -> usize {
        let resolutions = &self.zoom_resolutions;
        if resolutions.len() > 0 {
            if resolution >= resolutions[0] {
                return 0;
            } else if resolution <= resolutions[resolutions.len() - 1] {
                return resolutions.len() - 1;
            }

            let mut left = 0;
            let mut right = resolutions.len() - 1;
            let mut zoom = 0;

            while left <= right {
                let mid = left + (right - left) / 2;
                let mid_val = resolutions[mid];

                if mid_val <= resolution {
                    zoom = mid;
                }

                if mid_val > resolution {
                    left = mid + 1;
                } else {
                    right = mid - 1;
                }
            }

            return zoom;
        }

        0
    }

    pub fn get_closest_zoom(&self, resolution: f64) -> usize {
        let resolutions = &self.zoom_resolutions;
        if resolutions.len() > 0 {
            if resolution >= resolutions[0] {
                return 0;
            } else if resolution <= resolutions[resolutions.len() - 1] {
                return resolutions.len() - 1;
            }

            let mut left = 0;
            let mut right = resolutions.len() - 1;
            let mut zoom = 0;
            let mut closest_diff = f64::MAX;

            while left <= right {
                let mid = left + (right - left) / 2;
                let mid_val = resolutions[mid];

                let diff = (mid_val - resolution).abs();
                if diff < closest_diff {
                    zoom = mid;
                    closest_diff = diff;
                }

                if mid_val > resolution {
                    left = mid + 1;
                } else {
                    right = mid - 1;
                }
            }

            return zoom;
        }

        0
    }

    pub fn get_max_x_y(&self, zoom: usize) -> i32 {
        let base_res = self.get_resolution(0);
        let zoom_res = self.get_resolution(zoom);

        (base_res / zoom_res).ceil() as i32 - 1
    }

    pub fn get_resolution(&self, zoom: usize) -> f64 {
        if self.zoom_resolutions.len() == 0 {
            self.origin_x.abs() * 2.0 / self.tile_size as f64
        } else {
            let max_zoom = self.zoom_resolutions.len() - 1;
            if zoom > max_zoom {
                self.zoom_resolutions[max_zoom]
            } else {
                self.zoom_resolutions[zoom]
            }
        }
    }

    pub fn get_tile_bbox(&self, tile_id: &TileId) -> Option<Bbox> {
        if tile_id.z < self.zoom_resolutions.len() {
            let res = self.zoom_resolutions[tile_id.z];
            let tile_map_size = res * self.tile_size as f64;

            let xmin = self.origin_x + tile_id.x as f64 * tile_map_size;
            let xmax = xmin + tile_map_size;
            let ymax = self.origin_y - tile_id.y as f64 * tile_map_size;
            let ymin = ymax - tile_map_size;

            return Some(Bbox::new(xmin, ymin, xmax, ymax));
        }

        None
    }

    pub fn get_tile_id(&self, z: usize, coord: &Coord) -> Option<TileId> {
        if z < self.zoom_resolutions.len() {
            let res = self.zoom_resolutions[z];
            let tile_map_size = res * self.tile_size as f64;

            let x = ((coord.x - self.origin_x) / tile_map_size).floor() as i32;
            let y = ((self.origin_y - coord.y) / tile_map_size).floor() as i32;

            return Some(TileId { z, x, y });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling() {
        let tiling = Tiling::default();
        for i in 0..tiling.zoom_resolutions.len() - 1 {
            assert_eq!(
                tiling.zoom_resolutions[i],
                tiling.zoom_resolutions[i + 1] * 2.0,
            )
        }

        assert_eq!(tiling.get_closest_lower_zoom(200000.0), 0);
        assert_eq!(tiling.get_closest_lower_zoom(156543.03392804094), 0);
        assert_eq!(tiling.get_closest_lower_zoom(156543.0), 1);
        assert_eq!(tiling.get_closest_lower_zoom(100000.0), 1);
        assert_eq!(tiling.get_closest_lower_zoom(39000.0), 3);
        assert_eq!(tiling.get_closest_lower_zoom(0.6), 18);
        assert_eq!(tiling.get_closest_lower_zoom(0.0001), 23);

        assert_eq!(tiling.get_closest_zoom(200000.0), 0);
        assert_eq!(tiling.get_closest_zoom(156543.03392804094), 0);
        assert_eq!(tiling.get_closest_zoom(156543.0), 0);
        assert_eq!(tiling.get_closest_zoom(100000.0), 1);
        assert_eq!(tiling.get_closest_zoom(39000.0), 2);
        assert_eq!(tiling.get_closest_zoom(0.6), 18);
        assert_eq!(tiling.get_closest_lower_zoom(0.0001), 23);
    }
}
