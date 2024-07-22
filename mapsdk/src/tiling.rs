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
    }
}
