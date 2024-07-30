use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use geo::Rect;
use image::DynamicImage;
use nanoid::nanoid;
use tokio::sync::mpsc;

use crate::{
    env,
    event::Event,
    layer::{Layer, LayerType},
    map::{context::MapState, Map, MapOptions},
    render::{draw::image::ImageDrawable, Renderer},
    utils::{http::HttpClient, image::image_from_url},
};

pub struct ImageLayer {
    url: String,
    rect: Rect,
    options: ImageLayerOptions,

    name: String,
    event_sender: Option<mpsc::UnboundedSender<Event>>,

    image_id: String,
    image: Arc<RwLock<Option<DynamicImage>>>,
    image_requested: bool,
    image_updated: Arc<AtomicBool>,
}

impl ImageLayer {
    pub fn new(url: &str, rect: Rect, options: ImageLayerOptions) -> Self {
        Self {
            url: url.to_string(),
            rect,
            options,

            name: String::new(),
            event_sender: None,

            image_id: nanoid!(),
            image: Arc::new(RwLock::new(None)),
            image_requested: false,
            image_updated: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Layer for ImageLayer {
    fn r#type(&self) -> LayerType {
        LayerType::ImageLayer
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
        renderer: &mut Renderer,
    ) {
        if !self.image_requested {
            self.image_requested = true;

            env::spawn({
                let url = self.url.clone();
                let headers = self.options.headers.clone();
                let image = self.image.clone();
                let image_updated = self.image_updated.clone();
                let event_sender = self.event_sender.clone();

                async move {
                    let http_client = HttpClient::new();

                    match image_from_url(&http_client, &url, &headers).await {
                        Ok(img) => {
                            log::debug!("Image loaded from {}", url);

                            if let Ok(mut image) = image.write() {
                                image.replace(img);
                                image_updated.store(true, Ordering::SeqCst);

                                if let Some(event_sender) = event_sender {
                                    let _ = event_sender.send(Event::MapRequestRedraw);
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Load image from {} error: {}", url, err);
                        }
                    }
                }
            });
        }

        if self.image_updated.load(Ordering::SeqCst) {
            if let Ok(image) = self.image.read() {
                if let Some(image) = image.as_ref() {
                    let drawable = ImageDrawable::new(renderer, &image, &self.rect, self.options.z);
                    renderer.add_draw_item(&self.image_id, drawable.into());
                }
            }

            self.image_updated.store(false, Ordering::SeqCst);
        }
    }
}

pub struct ImageLayerOptions {
    headers: Vec<(String, String)>,
    z: f64,
}

impl ImageLayerOptions {
    pub fn with_headers(mut self, v: &Vec<(impl ToString, impl ToString)>) -> Self {
        self.headers = v
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    pub fn with_z(mut self, v: f64) -> Self {
        self.z = v;
        self
    }
}

impl Default for ImageLayerOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            z: 0.0,
        }
    }
}
