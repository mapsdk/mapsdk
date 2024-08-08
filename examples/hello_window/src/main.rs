extern crate mapsdk;

use std::time::Duration;

use geo::{line_string, point, polygon, Coord, MultiLineString, MultiPolygon, Rect};
use mapsdk::{
    feature::{Feature, Shape},
    layer::{
        feature_layer::{FeatureLayer, FeatureLayerOptions},
        image_layer::{ImageLayer, ImageLayerOptions},
        image_tiled_layer::{ImageTiledLayer, ImageTiledLayerOptions},
    },
    map::{Map, MapOptions, MapViewChange},
    render::{Renderer, RendererOptions, RendererType},
    utils::{color::Color, proj::lonlat_to_wm},
};
use rand::Rng;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct App {
    map: Map,
    motion: Motion,
    rng: rand::rngs::ThreadRng,
}

#[derive(Default)]
struct Motion {
    cursor_position: Option<Coord>,

    dragging: bool,
    drag_start_cursor_position: Option<Coord>,
    drag_start_map_center: Option<Coord>,

    rotating: bool,
    rotate_start_cursor_position: Option<Coord>,
    pitch_start_value: f64,
    yaw_start_value: f64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(
            Window::default_attributes()
                .with_title("MapSDK - Hello Window")
                .with_transparent(true)
                .with_inner_size(LogicalSize::new(800.0, 500.0)),
        ) {
            let renderer = pollster::block_on(Renderer::new(
                RendererType::Window(window.into()),
                &RendererOptions::default()
                    .with_background_color(self.map.options().background_color.clone().into()),
            ));
            self.map.set_renderer(renderer);

            let headers = vec![("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"),("Accept", "image/webp,image/apng,image/*,*/*;q=0.8"),("Accept-Encoding", "gzip, deflate, br")];

            let image_layer_options = ImageLayerOptions::default()
                .with_headers(&headers)
                .with_z(-100.0);
            let image_layer = ImageLayer::new(
                "http://a.tile.osm.org/0/0/0.png",
                Rect::new(
                    Coord {
                        x: -20037508.34278924,
                        y: -20037508.34278924,
                    },
                    Coord {
                        x: 20037508.34278924,
                        y: 20037508.34278924,
                    },
                ),
                image_layer_options,
            );
            // let _ = self.map.add_layer("image", Box::new(image_layer));

            let image_tiled_layer_options = ImageTiledLayerOptions::default()
                .with_headers(&headers)
                .with_url_subdomains(&vec!["a", "b", "c"]);
            let image_tiled_layer = ImageTiledLayer::new(
                "http://{s}.tile.osm.org/{z}/{x}/{y}.png",
                image_tiled_layer_options,
            );
            let _ = self
                .map
                .add_layer("image tiled", Box::new(image_tiled_layer));

            let feature_layer_options = FeatureLayerOptions::default();
            let mut feature_layer = FeatureLayer::new(feature_layer_options);
            feature_layer.add_feature(Feature::new(
                "0",
                Shape::Circle {
                    center: Coord { x: 0.0, y: 0.0 },
                    radius: 1000_000.0,
                },
                None,
            ));
            feature_layer.add_feature(Feature::new(
                "1",
                Shape::Geometry(point!(lonlat_to_wm(&Coord { x: 55.0, y: 0.0 }).unwrap()).into()),
                None,
            ));
            feature_layer.add_feature(Feature::new(
                "2",
                Shape::Geometry(
                    MultiLineString::new(vec![
                        line_string![
                            lonlat_to_wm(&Coord { x: 20.0, y: -5.0 }).unwrap(),
                            lonlat_to_wm(&Coord { x: 20.0, y: 5.0 }).unwrap(),
                        ],
                        line_string![
                            lonlat_to_wm(&Coord { x: 35.0, y: -5.0 }).unwrap(),
                            lonlat_to_wm(&Coord { x: 32.0, y: 0.0 }).unwrap(),
                            lonlat_to_wm(&Coord { x: 35.0, y: 5.0 }).unwrap(),
                        ],
                        line_string![
                            lonlat_to_wm(&Coord { x: 45.0, y: -5.0 }).unwrap(),
                            lonlat_to_wm(&Coord { x: 48.0, y: 0.0 }).unwrap(),
                            lonlat_to_wm(&Coord { x: 45.0, y: 5.0 }).unwrap(),
                        ],
                    ])
                    .into(),
                ),
                None,
            ));
            feature_layer.add_feature(Feature::new(
                "3",
                Shape::Geometry(
                    MultiPolygon::new(vec![
                        polygon!(
                            exterior: [
                                lonlat_to_wm(&Coord { x: -80.0, y: 6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -80.0, y: -6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -70.0, y: -6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -70.0, y: 6.0 }).unwrap(),
                            ],
                            interiors: [
                                [
                                    lonlat_to_wm(&Coord { x: -78.0, y: 4.0 }).unwrap(),
                                    lonlat_to_wm(&Coord { x: -78.0, y: -4.0 }).unwrap(),
                                    lonlat_to_wm(&Coord { x: -72.0, y: -4.0 }).unwrap(),
                                    lonlat_to_wm(&Coord { x: -72.0, y: 4.0 }).unwrap(),
                                ],
                            ],
                        ),
                        polygon!(
                            exterior: [
                                lonlat_to_wm(&Coord { x: -60.0, y: 6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -60.0, y: -6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -50.0, y: -6.0 }).unwrap(),
                                lonlat_to_wm(&Coord { x: -50.0, y: 6.0 }).unwrap(),
                            ],
                            interiors: [],
                        ),
                    ])
                    .into(),
                ),
                None,
            ));
            let _ = self.map.add_layer("feature", Box::new(feature_layer));
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
                        self.motion.rotating = true;
                        self.motion.rotate_start_cursor_position = None;
                    }
                }
                ElementState::Released => {
                    if button == MouseButton::Left {
                        self.motion.dragging = false;
                    } else if button == MouseButton::Right {
                        self.motion.rotating = false;
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.motion.cursor_position = Some(Coord {
                    x: position.x,
                    y: position.y,
                });

                if self.motion.dragging {
                    match self.motion.drag_start_cursor_position {
                        Some(drag_start_cursor_position) => {
                            if let Some(drag_start_map_center) = self.motion.drag_start_map_center {
                                let map_res = self.map.resolution().unwrap();
                                let map_yaw = self.map.yaw();
                                let dx = (position.x - drag_start_cursor_position.x) * map_res;
                                let dy = (drag_start_cursor_position.y - position.y) * map_res;
                                let cos_map_yaw = map_yaw.to_radians().cos();
                                let sin_map_yaw = map_yaw.to_radians().sin();
                                let new_center = drag_start_map_center
                                    - Coord {
                                        x: dx * cos_map_yaw - dy * sin_map_yaw,
                                        y: dy * cos_map_yaw + dx * sin_map_yaw,
                                    };
                                self.map.set_center(new_center);
                            }
                        }
                        None => {
                            self.motion.drag_start_cursor_position = Some(Coord {
                                x: position.x,
                                y: position.y,
                            });
                            self.motion.drag_start_map_center = self.map.center();
                        }
                    }
                } else if self.motion.rotating {
                    match self.motion.rotate_start_cursor_position {
                        Some(rotate_start_cursor_position) => {
                            let w = self.map.width().unwrap() as f64;
                            let h = self.map.height().unwrap() as f64;
                            let dx = position.x - rotate_start_cursor_position.x;
                            let dy = position.y - rotate_start_cursor_position.y;
                            self.map.set_pitch_yaw(
                                self.motion.pitch_start_value
                                    + (dy / h) * self.map.options().pitch_max,
                                self.motion.yaw_start_value - (dx / w) * 90.0,
                            )
                        }
                        None => {
                            self.motion.rotate_start_cursor_position = Some(Coord {
                                x: position.x,
                                y: position.y,
                            });
                            self.motion.pitch_start_value = self.map.pitch();
                            self.motion.yaw_start_value = self.map.yaw();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let coord = if let Some(cursor_position) = self.motion.cursor_position {
                    self.map.to_map(&cursor_position)
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        repeat,
                        physical_key,
                        ..
                    },
                ..
            } => {
                if state == ElementState::Pressed && !repeat {
                    match physical_key {
                        PhysicalKey::Code(KeyCode::KeyE) => {
                            self.map.ease_to(
                                &MapViewChange::default()
                                    .with_center(
                                        lonlat_to_wm(&Coord {
                                            x: -180.0 + 360.0 * self.rng.gen::<f64>(),
                                            y: -60.0 + 120.0 * self.rng.gen::<f64>(),
                                        })
                                        .unwrap(),
                                    )
                                    .with_pitch(60.0 * self.rng.gen::<f64>())
                                    .with_yaw(-60.0 + 120.0 * self.rng.gen::<f64>()),
                                Duration::from_millis(2000),
                            );
                        }
                        PhysicalKey::Code(KeyCode::KeyJ) => {
                            self.map.jump_to(
                                &MapViewChange::default()
                                    .with_center(
                                        lonlat_to_wm(&Coord {
                                            x: -180.0 + 360.0 * self.rng.gen::<f64>(),
                                            y: -60.0 + 120.0 * self.rng.gen::<f64>(),
                                        })
                                        .unwrap(),
                                    )
                                    .with_pitch(60.0 * self.rng.gen::<f64>())
                                    .with_yaw(-60.0 + 120.0 * self.rng.gen::<f64>()),
                            );
                        }
                        _ => (),
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
        map: Map::new(
            &MapOptions::default()
                .with_zoom_max(19)
                .with_background_color(Color::from_rgb(180, 180, 180)),
        ),
        motion: Motion::default(),
        rng: rand::thread_rng(),
    };
    let _ = event_loop.run_app(&mut app);
}
