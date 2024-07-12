extern crate mapsdk;

use mapsdk::{
    common::Color,
    map::{Map, MapOptions},
    render::{Renderer, RendererOptions, RendererType},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    map: Map,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = event_loop
            .create_window(Window::default_attributes().with_title("MapSDK - Hello Window"))
        {
            let background_color = self.map.get_options().get_background_color().clone();
            let renderer = pollster::block_on(Renderer::new(
                RendererType::Window(window.into()),
                &RendererOptions::default().with_background_color(background_color.into()),
            ));
            self.map.set_renderer(renderer);
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
