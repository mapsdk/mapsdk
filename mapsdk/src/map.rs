use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use nanoid::nanoid;
use tokio::sync::{mpsc, Mutex};

use crate::{
    common::Color, env::Env, event::Event, geo::Coord, layer::Layer, render::Renderer,
    tiling::Tiling,
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

    pub fn center(&self) -> Option<Coord> {
        Some(self.context.read().ok()?.state.center.clone())
    }

    pub fn height(&self) -> Option<u32> {
        Some(self.context.read().ok()?.renderer.as_ref()?.height())
    }

    pub fn map_to_screen(&self, map_coord: &Coord) -> Option<Coord> {
        let screen_center_x = self.context.read().ok()?.renderer.as_ref()?.width() as f64 / 2.0;
        let screen_center_y = self.context.read().ok()?.renderer.as_ref()?.height() as f64 / 2.0;

        let map_center = self.context.read().ok()?.state.center.clone();
        let map_res = self.resolution()?;

        let screen_dx = (map_coord.x - map_center.x) / map_res;
        let screen_dy = (map_center.y - map_coord.y) / map_res;

        Some(Coord::new(
            screen_center_x + screen_dx,
            screen_center_y + screen_dy,
        ))
    }

    pub fn options(&self) -> &MapOptions {
        &self.options
    }

    pub fn pitch(&self) -> f64 {
        self.context
            .read()
            .ok()
            .and_then(|context| Some(context.state.pitch))
            .unwrap_or(0.0)
    }

    pub fn redraw(&mut self) {
        self.request_redraw();
    }

    pub fn resolution(&self) -> Option<f64> {
        let zoom_res = self.context.read().ok()?.state.zoom_res;
        let map_res_ratio = self.context.read().ok()?.state.map_res_ratio;

        Some(zoom_res * map_res_ratio)
    }

    pub fn remove_layer(&mut self, id: &str) {
        if let Ok(mut context) = self.context.write() {
            if let Some(layer) = context.layers.get_mut(id) {
                (*layer).unset_event_sender();
            }

            context.layers.remove(id);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Ok(mut context) = self.context.write() {
            context.resize(width, height);
        }

        self.request_redraw();
    }

    pub fn screen_to_map(&self, screen_coord: &Coord) -> Option<Coord> {
        let screen_center_x = self.context.read().ok()?.renderer.as_ref()?.width() as f64 / 2.0;
        let screen_center_y = self.context.read().ok()?.renderer.as_ref()?.height() as f64 / 2.0;

        let screen_dx = screen_coord.x - screen_center_x;
        let screen_dy = screen_coord.y - screen_center_y;

        let map_center = self.context.read().ok()?.state.center.clone();
        Some(map_center + Coord::new(screen_dx, -screen_dy) * self.resolution()?)
    }

    pub fn set_center(&mut self, center: Coord) {
        if let Ok(mut context) = self.context.write() {
            context.state.center = center;
        }

        self.request_redraw();
    }

    pub fn set_pitch(&mut self, pitch: f64) {
        if let Ok(mut context) = self.context.write() {
            context.set_pitch(pitch.clamp(0.0, self.options.pitch_max));
        }

        self.request_redraw();
    }

    pub fn set_renderer(&mut self, renderer: Renderer) {
        if let Ok(mut context) = self.context.write() {
            context.renderer = Some(renderer);
        }
    }

    pub fn width(&self) -> Option<u32> {
        Some(self.context.read().ok()?.renderer.as_ref()?.width())
    }

    pub fn zoom_around(&mut self, coord: &Coord, scalar: f64) {
        let zoom_res_max = self.options.tiling.get_resolution(self.options.zoom_min);
        let zoom_res_min = self.options.tiling.get_resolution(self.options.zoom_max);

        if let Ok(mut context) = self.context.write() {
            let zoom_res = context.state.zoom_res;
            let new_zoom_res = (zoom_res / scalar).clamp(zoom_res_min, zoom_res_max);
            if new_zoom_res != zoom_res {
                context.state.zoom_res = new_zoom_res;

                let center = context.state.center;
                context.state.center = *coord + (center - *coord) * (new_zoom_res / zoom_res);
            } else if new_zoom_res == zoom_res_max && scalar < 1.0 {
                let center = context.state.center;
                let origin_center = self.options.center;
                context.state.center = center + (origin_center - center) * (1.0 - scalar).powf(0.5);
            }
        }

        self.request_redraw();
    }

    fn request_redraw(&self) {
        let _ = self.event_sender.send(Event::MapRequestRedraw);
    }
}

#[derive(Clone)]
pub struct MapOptions {
    pub background_color: Color,
    pub center: Coord,
    pub pitch: f64,
    pub pitch_max: f64,
    pub tiling: Tiling,
    pub world_copy: bool,
    pub zoom: usize,
    pub zoom_max: usize,
    pub zoom_min: usize,
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgba(0, 0, 0, 0.0),
            center: Coord::new(0.0, 0.0),
            pitch: 0.0,
            pitch_max: 80.0,
            tiling: Tiling::default(),
            world_copy: true,
            zoom: 0,
            zoom_max: 20,
            zoom_min: 0,
        }
    }
}

impl MapOptions {
    pub fn with_background_color(mut self, v: Color) -> Self {
        self.background_color = v;
        self
    }

    pub fn with_center(mut self, v: Coord) -> Self {
        self.center = v;
        self
    }

    pub fn with_pitch(mut self, v: f64) -> Self {
        self.pitch = v;
        self
    }

    pub fn with_pitch_max(mut self, v: f64) -> Self {
        self.pitch_max = v;
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
            "Redraw map, center: {:?}, zoom resolution: {:?}, map resolution ratio: {:?}, pitch: {:?}",
            self.state.center,
            self.state.zoom_res,
            self.state.map_res_ratio,
            self.state.pitch,
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

    fn set_pitch(&mut self, pitch: f64) {
        self.state.pitch = pitch;

        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_pitch(pitch, &self.state);
        }
    }

    fn with_center(mut self, v: Coord) -> Self {
        self.state.center = v;
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
    pub center: Coord,
    pub pitch: f64,
    pub map_res_ratio: f64,
    pub zoom: usize,
    pub zoom_res: f64,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            center: Coord::new(0.0, 0.0),
            pitch: 0.0,
            map_res_ratio: 1.0,
            zoom: 0,
            zoom_res: 1.0,
        }
    }
}
