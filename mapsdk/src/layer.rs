use crate::{
    event::Event,
    map::{context::MapState, Map, MapOptions},
    render::Renderer,
};

pub mod feature_layer;
pub mod image_layer;
pub mod image_tiled_layer;
pub mod vector_tiled_layer;

pub(crate) mod tiled;

pub trait Layer: Send + Sync {
    fn r#type(&self) -> LayerType;
    fn on_add_to_map(&mut self, map: &Map);
    fn on_remove_from_map(&mut self, map: &Map);
    fn set_name(&mut self, name: &str);
    fn update(&mut self, map_options: &MapOptions, map_state: &MapState, renderer: &mut Renderer);
}

pub enum LayerType {
    FeatureLayer,
    ImageLayer,
    ImageTiledLayer,
    VectorTiledLayer,
}
