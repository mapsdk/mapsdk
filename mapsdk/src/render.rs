use std::{collections::HashMap, time::Instant};

use glam::{Quat, Vec3};
use wgpu::*;

use crate::{
    map::context::MapState,
    render::{
        camera::Camera,
        draw::DrawItem,
        resources::{
            layout::{create_image_params_bgl, create_image_texture_bgl},
            pipeline::create_image_pipeline,
            texture::{create_depth_texture, create_texture_from_image},
        },
        targets::Window,
    },
    utils::size::PixelSize,
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
    draw_items: HashMap<String, DrawItem>,
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

                let depth_texture = create_depth_texture(&rendering_context, width, height);
                let depth_texture_view =
                    depth_texture.create_view(&TextureViewDescriptor::default());

                let rendering_resources = RenderingResources {
                    color_sampler: rendering_context.device.create_sampler(&SamplerDescriptor {
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Nearest,
                        mipmap_filter: FilterMode::Nearest,
                        ..Default::default()
                    }),
                    depth_texture_view,
                    image_pipeline: create_image_pipeline(&rendering_context),
                };

                let mut camera = Camera::default();
                camera.set_eye(Vec3::new(0.0, 0.0, height as f32 / 2.0));
                camera.set_aspect(width as f32 / height as f32);

                Self {
                    renderer_options: renderer_options.clone(),

                    rendering_size: PixelSize::new(width, height),
                    rendering_context,
                    rendering_resources,

                    camera,
                    draw_items: HashMap::new(),
                }
            }
        }
    }

    pub fn add_draw_item(&mut self, id: &str, draw_item: DrawItem) {
        self.draw_items.insert(id.to_string(), draw_item);
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn contains_draw_item(&self, id: &str) -> bool {
        self.draw_items.contains_key(id)
    }

    pub fn width(&self) -> u32 {
        self.rendering_size.width
    }

    pub fn height(&self) -> u32 {
        self.rendering_size.height
    }

    pub fn remove_draw_item(&mut self, id: &str) {
        self.draw_items.remove(id);
    }

    pub fn render(&mut self, map_state: &MapState) {
        let instant = Instant::now();

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
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(self.renderer_options.background_color),
                            store: StoreOp::Store,
                        },
                    })],
                    // depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    //     view: &self.rendering_resources.depth_texture_view,
                    //     depth_ops: Some(Operations {
                    //         load: LoadOp::Clear(1.0),
                    //         store: StoreOp::Store,
                    //     }),
                    //     stencil_ops: None,
                    // }),
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                for (_, draw_item) in &self.draw_items {
                    draw_item.draw(map_state, self, &mut render_pass);
                }
            }

            queue.submit(Some(command_encoder.finish()));
            surface_texture.present();
        }

        log::info!("Renderer::render elapsed: {:?}", instant.elapsed());
    }

    pub fn resize(&mut self, width: u32, height: u32, map_state: &MapState) {
        self.rendering_size.width = width;
        self.rendering_size.height = height;

        self.update_camera_size();
        self.update_camera_position(map_state.pitch, map_state.yaw);

        let RenderingContext {
            surface,
            adapter,
            device,
            ..
        } = &self.rendering_context;

        if let Some(config) = surface.get_default_config(&adapter, width, height) {
            surface.configure(&device, &config);
        }

        let depth_texture = create_depth_texture(&self.rendering_context, width, height);
        self.rendering_resources.depth_texture_view =
            depth_texture.create_view(&TextureViewDescriptor::default());

        self.render(&map_state);
    }

    pub fn set_pitch_yaw(&mut self, pitch: f64, yaw: f64, map_state: &MapState) {
        self.update_camera_position(pitch, yaw);
        self.render(&map_state);
    }

    fn update_camera_position(&mut self, pitch: f64, yaw: f64) {
        let pitch_rad = pitch.to_radians() as f32;
        let yaw_rad = yaw.to_radians() as f32;

        let h = self.rendering_size.height as f32 / 2.0;

        let v0 = Vec3::Z * h;
        let v1 = Quat::from_axis_angle(Vec3::X, pitch_rad) * v0;
        let v2 = Quat::from_axis_angle(Vec3::Z, yaw_rad) * v1;

        let u0 = Vec3::Y;
        let u1 = Quat::from_axis_angle(Vec3::Z, yaw_rad) * u0;

        self.camera.set_eye(v2);
        self.camera.set_up(u1);
    }

    fn update_camera_size(&mut self) {
        let width = self.rendering_size.width as f32;
        let height = self.rendering_size.height as f32;

        self.camera.set_aspect(width / height);
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
    color_sampler: Sampler,
    depth_texture_view: TextureView,
    image_pipeline: RenderPipeline,
}

pub(crate) struct RenderingContext {
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}
