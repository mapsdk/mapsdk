use wgpu::RenderPass;

use crate::render::{
    draw::{feature::FeatureDrawable, image::ImageDrawable, vector_tile::VectorTileDrawable},
    MapState, Renderer,
};

pub(crate) mod feature;
pub(crate) mod image;
pub(crate) mod vector_tile;

pub(crate) trait Drawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass);
}

pub enum DrawItem {
    Feature(FeatureDrawable),
    Image(ImageDrawable),
    VectorTile(VectorTileDrawable),
}

impl DrawItem {
    pub fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        match self {
            Self::Feature(drawable) => drawable.draw(map_state, renderer, render_pass),
            Self::Image(drawable) => drawable.draw(map_state, renderer, render_pass),
            Self::VectorTile(drawable) => drawable.draw(map_state, renderer, render_pass),
        }
    }
}
