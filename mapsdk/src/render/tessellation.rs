pub mod circle;

pub struct Tessellation {
    pub fill_vertices: Vec<[f32; 2]>,
    pub fill_indices: Vec<u16>,

    pub stroke_vertices: Vec<[f32; 5]>, // x, y, norm_x, norm_y, angle
    pub stroke_indices: Vec<u16>,
}

impl Tessellation {
    pub fn new() -> Self {
        Self {
            fill_vertices: Vec::new(),
            fill_indices: Vec::new(),

            stroke_vertices: Vec::new(),
            stroke_indices: Vec::new(),
        }
    }
}
