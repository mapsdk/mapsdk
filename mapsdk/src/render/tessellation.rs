pub mod circle;
pub mod geometry;

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
    pub vertices: Vec<[f32; 8]>,
    pub indices: Vec<u16>,
}
