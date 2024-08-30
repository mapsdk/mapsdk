use crate::utils::color::Color;

#[derive(Clone, Debug)]
pub struct ShapeStyles {
    pub fill_enabled: bool,
    pub fill_color: Color,
    pub stroke_enabled: bool,
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub symbol_size: f32,
    pub outline_align: OutlineAlign,
}

impl Default for ShapeStyles {
    fn default() -> Self {
        Self {
            fill_enabled: true,
            fill_color: Color::from_rgba(255, 0, 0, 0.7),
            stroke_enabled: true,
            stroke_color: Color::from_rgb(0, 0, 0),
            stroke_width: 1.8,
            symbol_size: 8.0,
            outline_align: OutlineAlign::Center,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutlineAlign {
    Center,
    Side,
}
