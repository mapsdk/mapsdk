use crate::{common::Color, render::Renderer};

pub struct Map {
    options: MapOptions,
    renderer: Option<Renderer>,
}

impl Map {
    pub fn new(options: &MapOptions) -> Self {
        Self {
            options: options.clone(),
            renderer: None,
        }
    }

    pub fn get_options(&self) -> &MapOptions {
        &self.options
    }

    pub fn get_width(&self) -> Option<u32> {
        match &self.renderer {
            Some(renderer) => Some(renderer.get_width()),
            _ => None,
        }
    }

    pub fn get_height(&self) -> Option<u32> {
        match &self.renderer {
            Some(renderer) => Some(renderer.get_height()),
            _ => None,
        }
    }

    pub fn redraw(&self) {
        if let Some(renderer) = &self.renderer {
            renderer.redraw();
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    pub fn set_renderer(&mut self, renderer: Renderer) {
        self.renderer = Some(renderer);
    }
}

#[derive(Clone)]
pub struct MapOptions {
    background_color: Color,
    world_copy: bool,
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgba(0, 0, 0, 0.0),
            world_copy: true,
        }
    }
}

impl MapOptions {
    pub fn get_background_color(&self) -> &Color {
        &self.background_color
    }

    pub fn with_background_color(mut self, v: Color) -> Self {
        self.background_color = v;
        self
    }

    pub fn with_world_copy(mut self, v: bool) -> Self {
        self.world_copy = v;
        self
    }
}
