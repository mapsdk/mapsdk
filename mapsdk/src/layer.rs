use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use image::DynamicImage;
use nanoid::nanoid;
use tokio::sync::mpsc;

use crate::{
    env::Env,
    event::Event,
    geo::Coord,
    render::{draw::image::ImageDrawable, Renderer},
    utils::image::image_from_url,
};

pub mod tiled;

pub trait Layer: Send + Sync {
    fn r#type(&self) -> LayerType;
    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Event>);
    fn unset_event_sender(&mut self);
    fn update(&mut self, env: &Env, renderer: &mut Renderer);
}

pub enum LayerType {
    ImageLayer,
    ImageTiledLayer,
    VectorTiledLayer,
}

pub struct ImageLayer {
    options: ImageLayerOptions,
    event_sender: Option<mpsc::UnboundedSender<Event>>,

    image_id: String,
    image: Arc<RwLock<Option<DynamicImage>>>,
    image_requested: bool,
    image_updated: Arc<AtomicBool>,
}

/// Use LeftTop/LeftBottom/RightTop/RightBottom to describe image coordinates.
#[derive(Clone, Copy)]
pub struct ImageCoords {
    pub lt: Coord,
    pub lb: Coord,
    pub rt: Coord,
    pub rb: Coord,
}

struct ImageLayerOptions {
    url: String,
    headers: Vec<(&'static str, &'static str)>,
    coords: ImageCoords,
}

impl ImageLayer {
    pub fn new(url: &str, headers: Vec<(&'static str, &'static str)>, coords: ImageCoords) -> Self {
        Self {
            options: ImageLayerOptions {
                url: url.to_string(),
                headers,
                coords,
            },
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

    fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Event>) {
        self.event_sender = Some(event_sender);
    }

    fn unset_event_sender(&mut self) {
        self.event_sender = None;
    }

    fn update(&mut self, env: &Env, renderer: &mut Renderer) {
        if !self.image_requested {
            self.image_requested = true;

            let http_client = env.get_http_client();
            let url = self.options.url.clone();
            let headers = self.options.headers.clone();
            let image = self.image.clone();
            let image_updated = self.image_updated.clone();

            let event_sender = self.event_sender.clone();
            env.spawn(async move {
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
            });
        }

        if self.image_updated.load(Ordering::SeqCst) {
            if let Ok(image) = self.image.read() {
                if let Some(image) = image.as_ref() {
                    let drawable = ImageDrawable::new(renderer, &image, &self.options.coords);
                    renderer.set_draw_item(&self.image_id, drawable.into());
                }
            }

            self.image_updated.store(false, Ordering::SeqCst);
        }
    }
}
