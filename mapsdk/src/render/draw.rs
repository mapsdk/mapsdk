use wgpu::RenderPass;

use crate::render::{draw::image::ImageDrawable, MapState, Renderer};

pub(crate) mod image;

pub(crate) trait Drawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass);
}

pub enum DrawItem {
    Image(ImageDrawable),
}

impl DrawItem {
    pub fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        match self {
            Self::Image(drawable) => drawable.draw(map_state, renderer, render_pass),
        }
    }
}
