use crate::utils::color::Color;

#[derive(Clone, Debug)]
pub struct ShapeStyles {
    pub symbol_size: f32,
    pub stroke_enabled: bool,
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_enabled: bool,
    pub fill_color: Color,
}

impl Default for ShapeStyles {
    fn default() -> Self {
        Self {
            symbol_size: 8.0,
            stroke_enabled: true,
            stroke_color: Color::from_rgb(0, 0, 0),
            stroke_width: 2.8,
            fill_enabled: true,
            fill_color: Color::from_rgba(255, 0, 0, 0.7),
        }
    }
}
