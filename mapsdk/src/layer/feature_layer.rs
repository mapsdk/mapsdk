use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::{
    event::Event,
    feature::{style::ShapeStyles, Feature, Features},
    layer::{Layer, LayerType},
    map::{context::MapState, Map, MapOptions},
    render::{draw::feature::FeatureDrawable, InterRenderers, MapRenderer},
};

pub struct FeatureLayer {
    options: FeatureLayerOptions,

    name: String,
    event_sender: Option<mpsc::UnboundedSender<Event>>,

    features: Arc<DashMap<String, Feature>>,
}

impl FeatureLayer {
    pub fn new(options: FeatureLayerOptions) -> Self {
        Self {
            options,

            name: String::new(),
            event_sender: None,

            features: Arc::new(DashMap::new()),
        }
    }

    pub fn add_feature(&mut self, feature: Feature) {
        self.features.insert(feature.id().to_string(), feature);

        if let Some(event_sender) = &self.event_sender {
            let _ = event_sender.send(Event::MapRequestRedraw);
        }
    }

    pub fn add_features_from_geojson(&mut self, geojson: &str) {
        if let Ok(features) = Features::from_geojson(geojson) {
            match features {
                Features::Single(feature) => {
                    self.features.insert(feature.id().to_string(), feature);
                }
                Features::Collection(features) => {
                    for feature in features {
                        self.features.insert(feature.id().to_string(), feature);
                    }
                }
            }

            if let Some(event_sender) = &self.event_sender {
                let _ = event_sender.send(Event::MapRequestRedraw);
            }
        }
    }
}

impl Layer for FeatureLayer {
    fn r#type(&self) -> LayerType {
        LayerType::FeatureLayer
    }

    fn on_add_to_map(&mut self, map: &Map) {
        self.event_sender = Some(map.event_sender.clone());
    }

    fn on_remove_from_map(&mut self, _map: &Map) {
        self.event_sender = None;
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn update(
        &mut self,
        _map_options: &MapOptions,
        _map_state: &MapState,
        map_renderer: &mut MapRenderer,
        _inter_renderers: &mut InterRenderers,
    ) {
        for pair in self.features.iter() {
            let feature_id = pair.key();
            let feature = pair.value();

            if !map_renderer.contains_layer_draw_item(&self.name, &feature_id) {
                let drawable = FeatureDrawable::new(
                    &map_renderer,
                    &feature,
                    self.options.z,
                    &self.options.shape_styles,
                );

                map_renderer.add_layer_draw_item(&self.name, &feature_id, drawable.into());
            }
        }
    }
}

pub struct FeatureLayerOptions {
    shape_styles: ShapeStyles,
    z: f64,
}

impl FeatureLayerOptions {
    pub fn with_shape_styles(mut self, v: ShapeStyles) -> Self {
        self.shape_styles = v;
        self
    }

    pub fn with_z(mut self, v: f64) -> Self {
        self.z = v;
        self
    }
}

impl Default for FeatureLayerOptions {
    fn default() -> Self {
        Self {
            shape_styles: ShapeStyles::default(),
            z: 0.0,
        }
    }
}
