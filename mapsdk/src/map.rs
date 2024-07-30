use std::{
    error::Error,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use geo::Coord;
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};

use crate::{
    env, event::Event, layer::Layer, map::context::MapContext, render::Renderer, tiling::Tiling,
    utils::color::Color,
};

pub(crate) mod context;

pub struct Map {
    pub options: MapOptions,

    pub(crate) context: Arc<Mutex<MapContext>>,
    pub(crate) context_redraw_seq: Arc<AtomicU64>,
    pub(crate) redraw_seq: Arc<AtomicU64>,
    pub(crate) event_sender: mpsc::UnboundedSender<Event>,

    event_handle: JoinHandle<()>,
    redraw_handle: JoinHandle<()>,
}

impl Drop for Map {
    fn drop(&mut self) {
        self.event_handle.abort();
    }
}

impl Map {
    pub fn new(options: &MapOptions) -> Self {
        let _ = env_logger::try_init();

        let context = Arc::new(Mutex::new(MapContext::new(options)));
        let context_redraw_seq = Arc::new(AtomicU64::new(0));
        let redraw_seq = Arc::new(AtomicU64::new(0));

        let (event_sender, mut event_receiver) = mpsc::unbounded_channel::<Event>();

        let event_handle = env::spawn({
            let redraw_seq = redraw_seq.clone();

            async move {
                loop {
                    let event = { event_receiver.recv().await };

                    if let Some(event) = event {
                        log::debug!("Event: {:?}", event);

                        match event {
                            Event::MapRequestRedraw => {
                                redraw_seq.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        });

        assert!(options.max_frame_rate > 0 && options.max_frame_rate <= 1000);
        let frame_interval = 1000 / options.max_frame_rate as u64;

        let redraw_handle = env::spawn({
            let context = context.clone();
            let context_redraw_seq = context_redraw_seq.clone();
            let redraw_seq = redraw_seq.clone();

            async move {
                loop {
                    if redraw_seq.load(Ordering::SeqCst)
                        == context_redraw_seq.load(Ordering::SeqCst)
                    {
                        sleep(Duration::from_millis(frame_interval)).await;
                    } else {
                        context_redraw_seq
                            .store(redraw_seq.load(Ordering::SeqCst), Ordering::SeqCst);

                        let now = Instant::now();

                        {
                            if let Ok(mut context) = context.lock() {
                                context.redraw();
                            }
                        }

                        let elapsed = now.elapsed().as_millis() as u64;
                        if frame_interval > elapsed {
                            sleep(Duration::from_millis(frame_interval - elapsed)).await;
                        }
                    }
                }
            }
        });

        Self {
            options: options.clone(),

            context,
            context_redraw_seq,
            redraw_seq,
            event_sender,

            event_handle,
            redraw_handle,
        }
    }

    pub fn add_layer(
        &mut self,
        name: &str,
        mut layer: Box<dyn Layer>,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(context) = self.context.lock() {
            if context.layers.contains_key(name) {
                return Err(format!("Layer {} already exists", name).into());
            }
        }

        layer.set_name(name);
        layer.on_add_to_map(self);

        {
            if let Ok(mut context) = self.context.lock() {
                context.layers.insert(name.to_string(), layer);
            }
        }

        self.request_redraw();

        Ok(())
    }

    pub fn center(&self) -> Option<Coord> {
        Some(self.context.lock().ok()?.state.center.clone())
    }

    pub fn height(&self) -> Option<u32> {
        Some(self.context.lock().ok()?.renderer.as_ref()?.height())
    }

    pub fn options(&self) -> &MapOptions {
        &self.options
    }

    pub fn pitch(&self) -> f64 {
        self.context
            .lock()
            .ok()
            .and_then(|context| Some(context.state.pitch))
            .unwrap_or(0.0)
    }

    pub fn redraw(&mut self) {
        self.request_redraw();
    }

    pub fn remove_layer(&mut self, name: &str) {
        {
            if let Ok(mut context) = self.context.lock() {
                if let Some(layer) = context.layers.get_mut(name) {
                    (*layer).on_remove_from_map(self);
                }

                context.layers.remove(name);
            }
        }

        self.request_redraw();
    }

    pub fn resolution(&self) -> Option<f64> {
        let zoom_res = self.context.lock().ok()?.state.zoom_res;
        let map_res_ratio = self.context.lock().ok()?.state.map_res_ratio;

        Some(zoom_res * map_res_ratio)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        {
            if let Ok(mut context) = self.context.lock() {
                context.resize(width, height);
            }
        }

        self.request_redraw();
    }

    pub fn set_center(&mut self, center: Coord) {
        {
            if let Ok(mut context) = self.context.lock() {
                context.set_center(center);
            }
        }

        self.request_redraw();
    }

    pub fn set_pitch_yaw(&mut self, pitch: f64, yaw: f64) {
        {
            if let Ok(mut context) = self.context.lock() {
                context.set_pitch_yaw(pitch.clamp(0.0, self.options.pitch_max), yaw);
            }
        }

        self.request_redraw();
    }

    pub fn set_renderer(&mut self, renderer: Renderer) {
        {
            if let Ok(mut context) = self.context.lock() {
                context.renderer = Some(renderer);
            }
        }

        self.request_redraw();
    }

    pub fn to_map(&self, screen_coord: &Coord) -> Option<Coord> {
        self.context.lock().ok()?.to_map(screen_coord)
    }

    pub fn to_screen(&self, map_coord: &Coord) -> Option<Coord> {
        self.context.lock().ok()?.to_screen(map_coord)
    }

    pub fn width(&self) -> Option<u32> {
        Some(self.context.lock().ok()?.renderer.as_ref()?.width())
    }

    pub fn yaw(&self) -> f64 {
        self.context
            .lock()
            .ok()
            .and_then(|context| Some(context.state.yaw))
            .unwrap_or(0.0)
    }

    pub fn zoom_around(&mut self, coord: &Coord, scalar: f64) {
        {
            if let Ok(mut context) = self.context.lock() {
                context.zoom_around(coord, scalar);
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
    pub max_frame_rate: usize,
    pub pitch: f64, // degree
    pub pitch_max: f64,
    pub tiling: Tiling,
    pub world_copy: bool,
    pub yaw: f64, // degree
    pub zoom: usize,
    pub zoom_max: usize,
    pub zoom_min: usize,
}

impl Default for MapOptions {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgba(0, 0, 0, 0.0),
            center: Coord { x: 0.0, y: 0.0 },
            max_frame_rate: 60,
            pitch: 0.0,
            pitch_max: 80.0,
            tiling: Tiling::default(),
            world_copy: true,
            yaw: 0.0,
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

    pub fn with_yaw(mut self, v: f64) -> Self {
        self.yaw = v;
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
