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
    env,
    event::Event,
    layer::Layer,
    map::context::MapContext,
    render::{InterRenderers, MapRenderer, MapRendererOptions, VectorTileRenderer},
    tiling::Tiling,
    utils::color::Color,
    Canvas,
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

    anim_handle: Option<JoinHandle<()>>,
}

impl Drop for Map {
    fn drop(&mut self) {
        self.event_handle.abort();
    }
}

impl Map {
    pub fn new(canvas: Canvas, options: &MapOptions) -> Self {
        let _ = env_logger::try_init();

        let map_renderer = pollster::block_on(MapRenderer::new(
            canvas,
            &MapRendererOptions::default()
                .with_background_color(options.background_color.clone().into()),
        ));

        let vector_tile_renderer = pollster::block_on(VectorTileRenderer::new());

        let inter_renderers = InterRenderers {
            vector_tile_renderer,
        };

        let context = Arc::new(Mutex::new(MapContext::new(
            options,
            map_renderer,
            inter_renderers,
        )));

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

            anim_handle: None,
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
                context.state.layers_order.push(name.to_string());
            }
        }

        self.request_redraw();

        Ok(())
    }

    pub fn center(&self) -> Option<Coord> {
        Some(self.context.lock().ok()?.state.center.clone())
    }

    /// Ease to the given view, with an animated transition.
    pub fn ease_to(&mut self, map_view_change: &MapViewChange, duration: Duration) {
        self.fly_to(map_view_change, duration, 0);
    }

    /// Fly up and down to the given view, with an animated transition.
    pub fn fly_to(
        &mut self,
        map_view_change: &MapViewChange,
        duration: Duration,
        zoom_factor: u32,
    ) {
        let frame_interval = 1000 / self.options.max_frame_rate as u128;

        let ticks = duration.as_millis() / frame_interval;
        if ticks <= 1 {
            self.jump_to(map_view_change);
            return;
        }

        self.cancel_anim();

        self.anim_handle = Some(env::spawn({
            let event_sender = self.event_sender.clone();
            let context = self.context.clone();

            let from_center = self.center();
            let from_zoom_res = self.zoom_res();
            let from_pitch = self.pitch();
            let from_yaw = self.yaw();

            let to_center = map_view_change.center.clone();
            let to_zoom_res = map_view_change.zoom_res;
            let to_pitch = map_view_change.pitch;
            let to_yaw = map_view_change.yaw;

            let zoom_up = 2.0_f64.powi(zoom_factor as i32) - 1.0;

            async move {
                if let Ok(mut context) = context.lock() {
                    context.animating = true;
                }

                let now = Instant::now();

                for i in 0..ticks {
                    if now.elapsed().as_millis() >= frame_interval * i {
                        continue;
                    }

                    let t = (i as f64 / ticks as f64).clamp(0.0, 1.0);
                    let x = 1.0 - (1.0 - t).powi(3); // ease
                    let y = 1.0 - (0.5 - x).abs() * 2.0; // fly

                    {
                        if let Ok(mut context) = context.lock() {
                            if let (Some(from_center), Some(to_center)) = (from_center, to_center) {
                                let center = from_center * (1.0 - x) + to_center * x;

                                context.set_center(center);
                            }

                            {
                                let to_zoom_res = to_zoom_res.unwrap_or(from_zoom_res);
                                let zoom_res = (from_zoom_res * (1.0 - x) + to_zoom_res * x)
                                    * (1.0 + zoom_up * y);

                                context.set_zoom_res(zoom_res, false);
                            }

                            if to_pitch.is_some() || to_yaw.is_some() {
                                let to_pitch = to_pitch.unwrap_or(from_pitch);
                                let to_yaw = to_yaw.unwrap_or(from_yaw);

                                let pitch = from_pitch * (1.0 - x) + to_pitch * x;
                                let yaw = from_yaw * (1.0 - x) + to_yaw * x;

                                context.set_pitch_yaw(pitch, yaw);
                            }
                        }
                    }

                    let _ = event_sender.send(Event::MapRequestRedraw);

                    sleep(Duration::from_millis(frame_interval as u64)).await;
                }

                {
                    if let Ok(mut context) = context.lock() {
                        if let Some(center) = to_center {
                            context.set_center(center);
                        }

                        if let Some(zoom_res) = to_zoom_res {
                            context.set_zoom_res(zoom_res, true);
                        }

                        if to_pitch.is_some() || to_yaw.is_some() {
                            let pitch = to_pitch.unwrap_or(context.state.pitch);
                            let yaw = to_yaw.unwrap_or(context.state.yaw);
                            context.set_pitch_yaw(pitch, yaw);
                        }
                    }
                }

                if let Ok(mut context) = context.lock() {
                    context.animating = false;
                }

                let _ = event_sender.send(Event::MapRequestRedraw);
            }
        }));
    }

    pub fn height(&self) -> Option<u32> {
        Some(self.context.lock().ok()?.map_renderer.height())
    }

    /// Jump to the given view, without an animated transition.
    pub fn jump_to(&mut self, map_view_change: &MapViewChange) {
        self.cancel_anim();

        {
            if let Ok(mut context) = self.context.lock() {
                if let Some(center) = map_view_change.center {
                    context.set_center(center);
                }

                if let Some(zoom_res) = map_view_change.zoom_res {
                    context.set_zoom_res(zoom_res, true);
                }

                if map_view_change.pitch.is_some() || map_view_change.yaw.is_some() {
                    let pitch = map_view_change.pitch.unwrap_or(context.state.pitch);
                    let yaw = map_view_change.yaw.unwrap_or(context.state.yaw);
                    context.set_pitch_yaw(pitch, yaw);
                }
            }
        }

        self.request_redraw();
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
                context.state.layers_order.retain(|x| *x != name);
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
        self.cancel_anim();

        {
            if let Ok(mut context) = self.context.lock() {
                context.set_center(center);
            }
        }

        self.request_redraw();
    }

    pub fn set_pitch_yaw(&mut self, pitch: f64, yaw: f64) {
        self.cancel_anim();

        {
            if let Ok(mut context) = self.context.lock() {
                context.set_pitch_yaw(pitch.clamp(0.0, self.options.pitch_max), yaw);
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
        Some(self.context.lock().ok()?.map_renderer.width())
    }

    pub fn yaw(&self) -> f64 {
        self.context
            .lock()
            .ok()
            .and_then(|context| Some(context.state.yaw))
            .unwrap_or(0.0)
    }

    pub fn zoom_around(&mut self, coord: &Coord, scalar: f64) {
        self.cancel_anim();

        {
            if let Ok(mut context) = self.context.lock() {
                context.zoom_around(coord, scalar);
            }
        }

        self.request_redraw();
    }

    pub fn zoom_res(&self) -> f64 {
        self.context
            .lock()
            .ok()
            .and_then(|context| Some(context.state.zoom_res))
            .unwrap_or(0.0)
    }

    fn cancel_anim(&mut self) {
        if let Some(anim_handle) = self.anim_handle.take() {
            anim_handle.abort();
        }

        {
            if let Ok(mut context) = self.context.lock() {
                context.animating = false;
            }
        }

        self.anim_handle = None;
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

#[derive(Default)]
pub struct MapViewChange {
    pub center: Option<Coord>,
    pub zoom_res: Option<f64>,
    pub pitch: Option<f64>,
    pub yaw: Option<f64>,
}

impl MapViewChange {
    pub fn with_center(mut self, v: Coord) -> Self {
        self.center = Some(v);
        self
    }

    pub fn with_zoom_res(mut self, v: f64) -> Self {
        self.zoom_res = Some(v);
        self
    }

    pub fn with_pitch(mut self, v: f64) -> Self {
        self.pitch = Some(v);
        self
    }

    pub fn with_yaw(mut self, v: f64) -> Self {
        self.yaw = Some(v);
        self
    }
}
