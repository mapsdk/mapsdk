use std::{collections::HashSet, sync::Arc};

use dashmap::{DashMap, DashSet};
use geo::Intersects;
use image::DynamicImage;
use moka::sync::Cache;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    env,
    layer::{
        tiled::{format_tile_url, tile_ids_in_view},
        Event, Layer, LayerType,
    },
    map::{context::MapState, Map, MapOptions},
    render::{draw::image::ImageDrawable, Renderer},
    tiling::TileId,
    utils::http::{HttpPool, HttpRequest, HttpResponse},
};

pub struct ImageTiledLayer {
    url_template: String, // http://{s}.tile.osm.org/{z}/{x}/{y}.png
    options: ImageTiledLayerOptions,

    name: String,
    event_sender: Option<mpsc::UnboundedSender<Event>>,

    tile_fetcher: Option<HttpPool<TileId>>,
    tile_response_handle: Option<JoinHandle<()>>,

    tile_images_cache: Cache<TileId, DynamicImage>,

    requesting_tile_ids: Arc<DashSet<TileId>>,
    tile_images: Arc<DashMap<TileId, DynamicImage>>,
}

impl ImageTiledLayer {
    pub fn new(url_template: &str, options: ImageTiledLayerOptions) -> Self {
        let cache_size = options.cache_size;

        Self {
            url_template: url_template.to_string(),
            options,

            name: String::new(),
            event_sender: None,

            tile_fetcher: None,
            tile_response_handle: None,

            tile_images_cache: Cache::new(cache_size),

            requesting_tile_ids: Arc::new(DashSet::new()),
            tile_images: Arc::new(DashMap::new()),
        }
    }

    fn format_draw_item_id(&self, tile_id: &TileId) -> String {
        format!("[{}]{}", self.name, tile_id.to_string())
    }
}

impl Layer for ImageTiledLayer {
    fn r#type(&self) -> LayerType {
        LayerType::ImageTiledLayer
    }

    fn on_add_to_map(&mut self, map: &Map) {
        if self.event_sender.is_some() {
            return;
        }

        self.event_sender = Some(map.event_sender.clone());

        let (tile_response_sender, mut tile_response_receiver) =
            mpsc::unbounded_channel::<HttpResponse<TileId>>();
        self.tile_fetcher = Some(HttpPool::new(self.options.concurrent, tile_response_sender));

        self.tile_response_handle = Some(env::spawn({
            let tile_images_cache = self.tile_images_cache.clone();
            let requesting_tile_ids = self.requesting_tile_ids.clone();
            let event_sender = self.event_sender.clone();

            async move {
                loop {
                    let http_response = { tile_response_receiver.recv().await };

                    if let Some(http_response) = http_response {
                        let tile_id = http_response.id.clone();
                        if let Ok(image) = http_response.image().await {
                            log::debug!("Image tile {} loaded", tile_id.to_string());
                            tile_images_cache.insert(tile_id.clone(), image);

                            requesting_tile_ids.remove(&tile_id);

                            if let Some(event_sender) = &event_sender {
                                let _ = event_sender.send(Event::MapRequestRedraw);
                            }
                        }
                    }
                }
            }
        }));
    }

    fn on_remove_from_map(&mut self, _map: &Map) {
        if self.event_sender.is_none() {
            return;
        }

        self.event_sender = None;

        self.tile_fetcher = None;

        if let Some(tile_response_handle) = &self.tile_response_handle {
            tile_response_handle.abort();
        }

        self.tile_response_handle = None;
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn update(&mut self, map_options: &MapOptions, map_state: &MapState, renderer: &mut Renderer) {
        let tile_ids = tile_ids_in_view(map_state, &map_options.tiling);
        let center_tile_id = map_options
            .tiling
            .get_tile_id(map_state.zoom, &map_state.center);

        // Cancel tile requestes that are no longer needed
        {
            let mut cancel_tile_ids: Vec<TileId> = Vec::new();
            for tile_id in self.requesting_tile_ids.iter() {
                if !tile_ids.contains(&tile_id) {
                    cancel_tile_ids.push(tile_id.clone());
                }
            }

            cancel_tile_ids.iter().for_each(|tile_id| {
                self.requesting_tile_ids.remove(tile_id);

                if let Some(tile_fetcher) = &self.tile_fetcher {
                    tile_fetcher.cancel(&tile_id);
                }
            });
        }

        // Load tile images from cache if possible
        tile_ids.iter().for_each(|tile_id| {
            if !self.tile_images.contains_key(tile_id) {
                if let Some(tile_image) = self.tile_images_cache.get(tile_id) {
                    self.tile_images.insert(tile_id.clone(), tile_image.clone());
                }
            }
        });

        // Load tile images from server
        {
            let mut load_tile_ids = tile_ids
                .iter()
                .filter(|tile_id| !self.tile_images.contains_key(&tile_id))
                .collect::<Vec<_>>();

            load_tile_ids.sort_by_key(|tile_id| {
                let dis = if let Some(center_tile_id) = &center_tile_id {
                    (center_tile_id.x - tile_id.x).abs() + (center_tile_id.y - tile_id.y).abs()
                } else {
                    0
                };

                dis
            });

            for tile_id in load_tile_ids {
                if self.requesting_tile_ids.contains(tile_id) {
                    continue;
                }
                self.requesting_tile_ids.insert(tile_id.clone());

                if let Some(tile_fetcher) = &self.tile_fetcher {
                    let url = format_tile_url(
                        &self.url_template,
                        tile_id.z,
                        tile_id.x,
                        tile_id.y,
                        &self.options.url_subdomains,
                    );

                    tile_fetcher.send(HttpRequest::Get {
                        id: tile_id.clone(),
                        url,
                        headers: self.options.headers.clone(),
                    });
                }
            }
        }

        // Clear tiles that are no longer needed
        {
            let mut dirty_tiles: HashSet<TileId> = HashSet::new();

            for pair in self.tile_images.iter() {
                let tile_id = pair.key();

                if let Some(bbox) = map_options.tiling.get_tile_bbox(&tile_id) {
                    if !bbox.to_polygon().intersects(map_state.view_bounds()) {
                        dirty_tiles.insert(tile_id.clone());
                    }
                }

                if tile_id.z == map_state.zoom {
                    if !tile_ids.contains(tile_id) {
                        dirty_tiles.insert(tile_id.clone());
                    }
                } else {
                    dirty_tiles.insert(tile_id.clone());
                }
            }

            // Keep resample tiles if possible
            for tile_id in &tile_ids {
                if !self.tile_images.contains_key(tile_id) {
                    let mut roll_up_resampled = false;

                    for level in 1..=5 {
                        if let Some(parent_tile_id) =
                            map_options.tiling.roll_up_tile_id(tile_id, level)
                        {
                            if self.tile_images.contains_key(&parent_tile_id) {
                                dirty_tiles.remove(&parent_tile_id);
                                roll_up_resampled = true;
                                break;
                            }
                        }
                    }

                    if roll_up_resampled {
                        continue;
                    }

                    let child_tile_ids = map_options.tiling.drill_down_tile_ids(tile_id, 1);
                    if child_tile_ids.len() > 0 {
                        for child_tile_id in child_tile_ids {
                            dirty_tiles.remove(&child_tile_id);
                        }
                    }
                }
            }

            for tile_id in dirty_tiles {
                self.tile_images.remove(&tile_id);

                let draw_item_id = self.format_draw_item_id(&tile_id);
                renderer.remove_draw_item(&draw_item_id);
            }
        }

        for pair in self.tile_images.iter() {
            let tile_id = pair.key();
            let image = pair.value();

            if let Some(bbox) = map_options.tiling.get_tile_bbox(&tile_id) {
                let draw_item_id = self.format_draw_item_id(&tile_id);
                if !renderer.contains_draw_item(&draw_item_id) {
                    let drawable = ImageDrawable::new(renderer, &image, &bbox, self.options.z);
                    renderer.add_draw_item(&draw_item_id, drawable.into());
                }
            }
        }
    }
}

pub struct ImageTiledLayerOptions {
    cache_size: u64,
    concurrent: usize,
    headers: Vec<(String, String)>,
    url_subdomains: Option<Vec<String>>,
    z: f64,
}

impl ImageTiledLayerOptions {
    pub fn with_cache_size(mut self, v: u64) -> Self {
        self.cache_size = v;
        self
    }

    pub fn with_concurrent(mut self, v: usize) -> Self {
        self.concurrent = v;
        self
    }

    pub fn with_headers(mut self, v: &Vec<(impl ToString, impl ToString)>) -> Self {
        self.headers = v
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    pub fn with_url_subdomains(mut self, v: &Vec<impl ToString>) -> Self {
        self.url_subdomains = Some(v.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_z(mut self, v: f64) -> Self {
        self.z = v;
        self
    }
}

impl Default for ImageTiledLayerOptions {
    fn default() -> Self {
        Self {
            cache_size: 512,
            concurrent: 8,
            headers: Vec::new(),
            url_subdomains: None,
            z: 0.0,
        }
    }
}
