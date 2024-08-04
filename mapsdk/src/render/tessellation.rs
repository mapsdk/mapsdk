pub mod circle;
pub mod line_string;
pub mod polygon;

pub struct Tessellations {
    pub fills: Vec<FillVertexIndex>,
    pub strokes: Vec<StrokeVertexIndex>,
}

impl Tessellations {
    pub fn new() -> Self {
        Self {
            fills: Vec::new(),
            strokes: Vec::new(),
        }
    }
}

pub struct FillVertexIndex {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
}

pub struct StrokeVertexIndex {
    pub vertices: Vec<[f32; 5]>, // x, y, norm_x, norm_y, angle
    pub indices: Vec<u16>,
}
