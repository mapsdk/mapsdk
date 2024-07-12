pub struct Window {
    handle: Box<dyn wgpu::WindowHandle>,
    width: u32,
    height: u32,
}

impl From<winit::window::Window> for Window {
    fn from(winit_window: winit::window::Window) -> Self {
        let size = winit_window.inner_size();

        Self {
            handle: Box::new(winit_window),
            width: size.width,
            height: size.height,
        }
    }
}

impl Window {
    pub fn get_handle(self) -> Box<dyn wgpu::WindowHandle> {
        self.handle
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}
