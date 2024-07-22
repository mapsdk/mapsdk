extern crate mapsdk;

use mapsdk::{
    common::Color,
    geo::Coord,
    layer::{ImageCoords, ImageLayer},
    map::{Map, MapOptions},
    render::{Renderer, RendererOptions, RendererType},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    map: Map,
    motion: Motion,
}

#[derive(Default)]
struct Motion {
    cursor_position: Option<Coord>,

    dragging: bool,
    drag_start_cursor_position: Option<Coord>,
    drag_start_map_center: Option<Coord>,

    pitching: bool,
    pitch_start_cursor_position: Option<Coord>,
    pitch_start_value: f64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(
            Window::default_attributes()
                .with_title("MapSDK - Hello Window")
                .with_inner_size(LogicalSize::new(800.0, 500.0)),
        ) {
            let background_color = self.map.options().background_color.clone();
            let renderer = pollster::block_on(Renderer::new(
                RendererType::Window(window.into()),
                &RendererOptions::default().with_background_color(background_color.into()),
            ));
            self.map.set_renderer(renderer);

            let headers = vec![("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"),("Accept", "image/webp,image/apng,image/*,*/*;q=0.8"),("Accept-Encoding", "gzip, deflate, br")];

            let layer = ImageLayer::new(
                "http://a.tile.osm.org/0/0/0.png",
                headers,
                ImageCoords {
                    lt: Coord::new(-20037508.34278924, 20037508.34278924),
                    lb: Coord::new(-20037508.34278924, -20037508.34278924),
                    rt: Coord::new(20037508.34278924, 20037508.34278924),
                    rb: Coord::new(20037508.34278924, -20037508.34278924),
                },
            );
            let _ = self.map.add_layer(Box::new(layer));

            self.map.redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.map.redraw();
            }
            WindowEvent::Resized(size) => {
                self.map.resize(size.width, size.height);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if button == MouseButton::Left {
                        self.motion.dragging = true;
                        self.motion.drag_start_cursor_position = None;
                    } else if button == MouseButton::Right {
                        self.motion.pitching = true;
                        self.motion.pitch_start_cursor_position = None;
                    }
                }
                ElementState::Released => {
                    if button == MouseButton::Left {
                        self.motion.dragging = false;
                    } else if button == MouseButton::Right {
                        self.motion.pitching = false;
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.motion.cursor_position = Some(Coord::new(position.x, position.y));

                if self.motion.dragging {
                    match self.motion.drag_start_cursor_position {
                        Some(drag_start_cursor_position) => {
                            if let Some(drag_start_map_center) = self.motion.drag_start_map_center {
                                let map_res = self.map.resolution().unwrap();
                                let dx = (position.x - drag_start_cursor_position.x) * map_res;
                                let dy = (drag_start_cursor_position.y - position.y) * map_res;
                                let new_center = drag_start_map_center - Coord::new(dx, dy);
                                self.map.set_center(new_center);
                            }
                        }
                        None => {
                            self.motion.drag_start_cursor_position =
                                Some(Coord::new(position.x, position.y));
                            self.motion.drag_start_map_center = self.map.center();
                        }
                    }
                } else if self.motion.pitching {
                    match self.motion.pitch_start_cursor_position {
                        Some(pitch_start_cursor_position) => {
                            let h = self.map.height().unwrap() as f64;
                            let dy = position.y - pitch_start_cursor_position.y;
                            let factor = dy / h;
                            self.map.set_pitch(
                                self.motion.pitch_start_value
                                    + factor * self.map.options().pitch_max,
                            )
                        }
                        None => {
                            self.motion.pitch_start_cursor_position =
                                Some(Coord::new(position.x, position.y));
                            self.motion.pitch_start_value = self.map.pitch();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let coord = if let Some(cursor_position) = self.motion.cursor_position {
                    self.map.screen_to_map(&cursor_position)
                } else {
                    self.map.center()
                };

                if let Some(coord) = coord {
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            let scalar = 2.0_f64.powf((y / 20.0).into());
                            self.map.zoom_around(&coord, scalar);
                        }
                        MouseScrollDelta::PixelDelta(p) => {
                            let scalar = 2.0_f64.powf(p.y / 20.0);
                            self.map.zoom_around(&coord, scalar);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        map: Map::new(&MapOptions::default().with_background_color(Color::from_rgb(180, 180, 180))),
        motion: Motion::default(),
    };
    let _ = event_loop.run_app(&mut app);
}
