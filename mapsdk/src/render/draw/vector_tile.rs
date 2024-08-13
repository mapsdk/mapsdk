use geo::Rect;
use wgpu::*;

use crate::{
    feature::style::ShapeStyles,
    render::{
        draw::{Drawable, VectorTileFeatureDrawable},
        DrawItem, MapState, Renderer,
    },
    vector_tile::VectorTile,
};

pub struct VectorTileDrawable {
    layers_feature_drawables: Vec<Vec<VectorTileFeatureDrawable>>,
}

impl VectorTileDrawable {
    pub fn new(
        renderer: &Renderer,
        vector_tile: &VectorTile,
        tile_bbox: &Rect,
        z: f64,
        layers_shape_styles: &Vec<(String, ShapeStyles)>,
    ) -> Self {
        let mut layers_feature_drawables: Vec<Vec<VectorTileFeatureDrawable>> = Vec::new();

        for (layer_name, shape_styles) in layers_shape_styles {
            if let Some(layer) = &vector_tile.layers.get(layer_name) {
                let mut layer_feature_drawables: Vec<VectorTileFeatureDrawable> = Vec::new();
                for feature in &layer.features {
                    let feature_drawable = VectorTileFeatureDrawable::new(
                        renderer,
                        &feature,
                        tile_bbox,
                        z,
                        &shape_styles,
                    );
                    layer_feature_drawables.push(feature_drawable);
                }

                layers_feature_drawables.push(layer_feature_drawables);
            }
        }

        Self {
            layers_feature_drawables,
        }
    }
}

impl Drawable for VectorTileDrawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        for layer_feature_drawables in &self.layers_feature_drawables {
            for feature_drawable in layer_feature_drawables {
                feature_drawable.draw(map_state, renderer, render_pass);
            }
        }
    }
}

impl Into<DrawItem> for VectorTileDrawable {
    fn into(self) -> DrawItem {
        DrawItem::VectorTile(self)
    }
}
