use std::ops::{Add, Div, Mul, Sub};

const COORD_PRECISION: f64 = 1e-8;

#[derive(Clone, Copy, Debug)]
pub struct Bbox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
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
    pub x: f64,
    pub y: f64,
}

impl Coord {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < COORD_PRECISION && (self.y - other.y).abs() < COORD_PRECISION
    }
}

impl Add for Coord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Div<f64> for Coord {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Mul<f64> for Coord {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Sub for Coord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
