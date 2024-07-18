const COORD_PRECISION: f64 = 1e-8;

#[derive(Clone, Copy, Debug)]
pub struct Bbox {
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
}

impl Bbox {
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        Self {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    pub fn xmin(&self) -> f64 {
        self.xmin
    }

    pub fn ymin(&self) -> f64 {
        self.ymin
    }

    pub fn xmax(&self) -> f64 {
        self.xmax
    }

    pub fn ymax(&self) -> f64 {
        self.ymax
    }
}

impl PartialEq for Bbox {
    fn eq(&self, other: &Self) -> bool {
        (self.xmin - other.xmin).abs() < COORD_PRECISION
            && (self.ymin - other.ymin).abs() < COORD_PRECISION
            && (self.xmax - other.xmax).abs() < COORD_PRECISION
            && (self.ymax - other.ymax).abs() < COORD_PRECISION
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coord {
    x: f64,
    y: f64,
}

impl Coord {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < COORD_PRECISION && (self.y - other.y).abs() < COORD_PRECISION
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
