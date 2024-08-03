use crate::utils::color::Color;

#[derive(Clone, Debug)]
pub struct ShapeStyles {
    pub stroke_enabled: bool,
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_enabled: bool,
    pub fill_color: Color,
}

impl Default for ShapeStyles {
    fn default() -> Self {
        Self {
            stroke_enabled: true,
            stroke_color: Color::from_rgb(0, 0, 0),
            stroke_width: 2.0,
            fill_enabled: true,
            fill_color: Color::from_rgb(255, 0, 0),
        }
    }
}
