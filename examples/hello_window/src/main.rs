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
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    map: Map,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(
            Window::default_attributes()
                .with_title("MapSDK - Hello Window")
                .with_inner_size(LogicalSize::new(800.0, 500.0)),
        ) {
            let background_color = self.map.options().background_color().clone();
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
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    let scale = 2.0_f64.powf((y / 20.0).into());
                    self.map.zoom_by(scale);
                }
                MouseScrollDelta::PixelDelta(p) => {
                    let scale = 2.0_f64.powf(p.y / 20.0);
                    self.map.zoom_by(scale);
                }
            },
            _ => (),
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        map: Map::new(&MapOptions::default().with_background_color(Color::from_rgb(0, 255, 255))),
    };
    let _ = event_loop.run_app(&mut app);
}
