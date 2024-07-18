use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use nanoid::nanoid;
use tokio::sync::{mpsc, Mutex};

use crate::{
    common::Color,
    env::Env,
    event::Event,
    geo::{Coord, Tiling},
    layer::Layer,
    render::Renderer,
};

pub struct Map {
    options: MapOptions,
    context: Arc<RwLock<MapContext>>,

    env: Arc<Env>,
    event_sender: mpsc::UnboundedSender<Event>,
}

impl Map {
    pub fn new(options: &MapOptions) -> Self {
        let zoom = options.zoom;
        let zoom_res = options.tiling.get_resolution(zoom);
        let context = Arc::new(RwLock::new(
            MapContext::new(options)
                .with_center(options.center.clone())
                .with_tiling(options.tiling.clone())
                .with_zoom(zoom)
                .with_zoom_res(zoom_res),
        ));

        let env = Arc::new(Env::new());
        let (event_sender, event_receiver) = mpsc::unbounded_channel::<Event>();

        let shared_event_receiver = Arc::new(Mutex::new(event_receiver));
        let shared_context = context.clone();
        let shared_env = env.clone();

        env.spawn(async move {
            while let Some(event) = shared_event_receiver.lock().await.recv().await {
                log::debug!("Event: {:?}", event);

                match event {
                    Event::MapRequestRedraw => {
                        if let Ok(mut context) = shared_context.write() {
                            context.redraw(&shared_env);
                        }
                    }
                }
            }
        });

        Self {
            options: options.clone(),
            context,

            env,
            event_sender,
        }
    }

    pub fn add_layer(&mut self, mut layer: Box<dyn Layer>) -> String {
        let id = nanoid!();

        layer.set_event_sender(self.event_sender.clone());
        if let Ok(mut context) = self.context.write() {
            context.layers.insert(id.clone(), layer);
        }

        id
    }

    pub fn remove_layer(&mut self, id: &str) {
        if let Ok(mut context) = self.context.write() {
            if let Some(layer) = context.layers.get_mut(id) {
                (*layer).unset_event_sender();
            }

            context.layers.remove(id);
        }
    }

    pub fn options(&self) -> &MapOptions {
        &self.options
    }

    pub fn zoom_by(&mut self, scale: f64) {
        let zoom_res_max = self.options.tiling.get_resolution(self.options.zoom_min);
        let zoom_res_min = self.options.tiling.get_resolution(self.options.zoom_max);

        if let Ok(mut context) = self.context.write() {
            let res = context.state.zoom_res / scale;
            context.state.zoom_res = res.clamp(zoom_res_min, zoom_res_max);

            self.request_redraw();
        }
    }

    pub fn width(&self) -> Option<u32> {
        if let Ok(context) = self.context.read() {
            if let Some(renderer) = &context.renderer {
                return Some((*renderer).width());
            }
        }

        None
    }

    pub fn height(&self) -> Option<u32> {
        if let Ok(context) = self.context.read() {
            if let Some(renderer) = &context.renderer {
                return Some((*renderer).height());
            }
        }

        None
    }

    pub fn redraw(&mut self) {
        self.request_redraw();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Ok(mut context) = self.context.write() {
            context.resize(width, height);
        }

        self.request_redraw();
    }

    pub fn set_renderer(&mut self, renderer: Renderer) {
        if let Ok(mut context) = self.context.write() {
            context.renderer = Some(renderer);
        }
    }

    fn request_redraw(&self) {
        let _ = self.event_sender.send(Event::MapRequestRedraw);
    }
}

#[derive(Clone)]
pub struct MapOptions {
    background_color: Color,
    center: Coord,
    tiling: Tiling,
    world_copy: bool,
    zoom: usize,
    zoom_max: usize,
    zoom_min: usize,
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgba(0, 0, 0, 0.0),
            center: Coord::new(0.0, 0.0),
            tiling: Tiling::default(),
            world_copy: true,
            zoom: 0,
            zoom_max: 20,
            zoom_min: 0,
        }
    }
}

impl MapOptions {
    pub fn background_color(&self) -> &Color {
        &self.background_color
    }

    pub fn center(&self) -> &Coord {
        &self.center
    }

    pub fn world_copy(&self) -> bool {
        self.world_copy
    }

    pub fn zoom(&self) -> usize {
        self.zoom
    }

    pub fn zoom_max(&self) -> usize {
        self.zoom_max
    }

    pub fn zoom_min(&self) -> usize {
        self.zoom_min
    }

    pub fn with_background_color(mut self, v: Color) -> Self {
        self.background_color = v;
        self
    }

    pub fn with_center(mut self, v: Coord) -> Self {
        self.center = v;
        self
    }

    pub fn with_world_copy(mut self, v: bool) -> Self {
        self.world_copy = v;
        self
    }

    pub fn with_zoom(mut self, v: usize) -> Self {
        self.zoom = v;
        self
    }

    pub fn with_zoom_max(mut self, v: usize) -> Self {
        self.zoom_max = v;
        self
    }

    pub fn with_zoom_min(mut self, v: usize) -> Self {
        self.zoom_min = v;
        self
    }
}

struct MapContext {
    map_options: MapOptions,
    state: MapState,

    layers: BTreeMap<String, Box<dyn Layer>>,
    renderer: Option<Renderer>,
}

impl MapContext {
    fn new(map_options: &MapOptions) -> Self {
        Self {
            map_options: map_options.clone(),
            state: MapState::default(),

            layers: BTreeMap::new(),
            renderer: None,
        }
    }

    fn redraw(&mut self, env: &Env) {
        log::debug!(
            "Redraw map, center: {:?}, zoom resolution: {:?}, map resolution ratio: {:?}",
            self.state.center,
            self.state.zoom_res,
            self.state.map_res_ratio,
        );

        if let Some(renderer) = self.renderer.as_mut() {
            for (id, layer) in &mut self.layers {
                log::debug!("Update layer {}", id);
                layer.update(env, renderer);
            }

            renderer.render(&self.state);
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.state.map_res_ratio =
            self.map_options.tiling.tile_size as f64 / width.min(height) as f64;

        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(width, height, &self.state);
        }
    }

    fn with_center(mut self, v: Coord) -> Self {
        self.state.center = v;
        self
    }

    fn with_tiling(mut self, v: Tiling) -> Self {
        self.state.tiling = v;
        self
    }

    fn with_zoom(mut self, v: usize) -> Self {
        self.state.zoom = v;
        self
    }

    fn with_zoom_res(mut self, v: f64) -> Self {
        self.state.zoom_res = v;
        self
    }
}

pub struct MapState {
    center: Coord,
    map_res_ratio: f64,
    zoom: usize,
    zoom_res: f64,

    tiling: Tiling,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            center: Coord::new(0.0, 0.0),
            map_res_ratio: 1.0,
            zoom: 0,
            zoom_res: 1.0,

            tiling: Tiling::default(),
        }
    }
}

impl MapState {
    pub fn center(&self) -> &Coord {
        &self.center
    }

    pub fn map_res_ratio(&self) -> f64 {
        self.map_res_ratio
    }

    pub fn zoom(&self) -> usize {
        self.zoom
    }

    pub fn zoom_res(&self) -> f64 {
        self.zoom_res
    }
}
