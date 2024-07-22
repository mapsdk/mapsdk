use dashmap::DashMap;
use wgpu::*;

use crate::{
    common::PixelSize,
    layer::ImageCoords,
    map::MapState,
    render::{
        camera::Camera,
        draw::DrawItem,
        resources::{
            layout::{create_image_params_bgl, create_image_texture_bgl},
            pipeline::create_image_pipeline,
            texture::create_texture_from_image,
        },
        targets::Window,
    },
};

pub(crate) mod camera;
pub(crate) mod draw;
pub(crate) mod resources;
pub(crate) mod targets;

pub struct Renderer {
    renderer_options: RendererOptions,

    rendering_size: PixelSize,
    rendering_context: RenderingContext,
    rendering_resources: RenderingResources,

    camera: Camera,
    draw_items: DashMap<String, DrawItem>,
}

impl Renderer {
    pub async fn new(renderer_type: RendererType, renderer_options: &RendererOptions) -> Self {
        match renderer_type {
            RendererType::Window(window) => {
                let width = window.width();
                let height = window.height();

                let instance = Instance::default();
                let surface = instance
                    .create_surface(window.handle())
                    .expect("Failed to create surface");
                let adapter = instance
                    .request_adapter(&RequestAdapterOptions {
                        power_preference: PowerPreference::default(),
                        force_fallback_adapter: false,
                        compatible_surface: Some(&surface),
                    })
                    .await
                    .expect("Failed to find adapter");
                let (device, queue) = adapter
                    .request_device(
                        &DeviceDescriptor {
                            required_features: Features::empty(),
                            required_limits: if cfg!(target_arch = "wasm32") {
                                Limits::downlevel_webgl2_defaults()
                                    .using_resolution(adapter.limits())
                            } else {
                                Limits::default().using_resolution(adapter.limits())
                            },
                            memory_hints: Default::default(),
                            label: None,
                        },
                        None,
                    )
                    .await
                    .expect("Failed to find device");
                if let Some(config) = surface.get_default_config(&adapter, width, height) {
                    surface.configure(&device, &config);
                }

                let rendering_context = RenderingContext {
                    surface,
                    adapter,
                    device,
                    queue,
                };

                let rendering_resources = RenderingResources {
                    image_pipeline: create_image_pipeline(&rendering_context),
                    image_sampler: rendering_context.device.create_sampler(&SamplerDescriptor {
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Nearest,
                        mipmap_filter: FilterMode::Nearest,
                        ..Default::default()
                    }),
                };

                let mut camera = Camera::default();
                camera.set_eye(0.0, 0.0, height as f32 / 2.0);
                camera.set_aspect(width as f32 / height as f32);

                Self {
                    renderer_options: renderer_options.clone(),

                    rendering_size: PixelSize::new(width, height),
                    rendering_context,
                    rendering_resources,

                    camera,
                    draw_items: DashMap::new(),
                }
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.rendering_size.width
    }

    pub fn height(&self) -> u32 {
        self.rendering_size.height
    }

    pub fn render(&mut self, map_state: &MapState) {
        let RenderingContext {
            surface,
            device,
            queue,
            ..
        } = &self.rendering_context;

        if let Ok(surface_texture) = surface.get_current_texture() {
            let surface_view = surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default());
            let mut command_encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });
            {
                let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(self.renderer_options.background_color),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                self.draw_items.iter_mut().for_each(|draw_item| {
                    (*draw_item).draw(map_state, self, &mut render_pass);
                });
            }

            queue.submit(Some(command_encoder.finish()));
            surface_texture.present();
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, map_state: &MapState) {
        self.rendering_size.width = width;
        self.rendering_size.height = height;

        self.camera.set_eye(0.0, 0.0, height as f32 / 2.0);
        self.camera.set_aspect(width as f32 / height as f32);

        let RenderingContext {
            surface,
            adapter,
            device,
            ..
        } = &self.rendering_context;

        if let Some(config) = surface.get_default_config(&adapter, width, height) {
            surface.configure(&device, &config);
        }

        self.render(&map_state);
    }

    pub fn set_draw_item(&mut self, id: &str, draw_item: DrawItem) {
        self.draw_items.insert(id.to_string(), draw_item);
    }

    pub fn set_pitch(&mut self, pitch: f64, map_state: &MapState) {
        let z = self.rendering_size.height as f32 / 2.0;
        let y = z * pitch.to_radians().tan() as f32;

        self.camera.set_eye(0.0, -y, z);

        self.render(&map_state);
    }
}

#[derive(Clone)]
pub struct RendererOptions {
    background_color: Color,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            background_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }
    }
}

impl RendererOptions {
    pub fn with_background_color(mut self, v: Color) -> Self {
        self.background_color = v;
        self
    }
}

pub enum RendererType {
    Window(Window),
}

pub(crate) struct RenderingResources {
    image_pipeline: RenderPipeline,
    image_sampler: Sampler,
}

pub(crate) struct RenderingContext {
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}
