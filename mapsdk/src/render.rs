use std::borrow::Cow;

use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState,
    Instance, Limits, LoadOp, Operations, PowerPreference, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderSource, StoreOp, Surface, TextureViewDescriptor, VertexState,
};

use crate::render::targets::Window;

pub mod targets;

pub struct Renderer {
    renderer_options: RendererOptions,

    rendering_size: RenderingSize,
    rendering_target: RenderingTarget,
}

impl Renderer {
    pub fn redraw(&self) {
        let RenderingTarget {
            surface,
            device,
            queue,
            render_pipeline,
            ..
        } = &self.rendering_target;

        if let Ok(frame) = surface.get_current_texture() {
            let view = frame.texture.create_view(&TextureViewDescriptor::default());
            let mut encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
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
                render_pass.set_pipeline(&render_pipeline);
                render_pass.draw(0..3, 0..1);
            }

            queue.submit(Some(encoder.finish()));
            frame.present();
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.rendering_size.width = width;
        self.rendering_size.height = height;

        let RenderingTarget {
            surface,
            adapter,
            device,
            ..
        } = &self.rendering_target;

        if let Some(config) = surface.get_default_config(&adapter, width, height) {
            surface.configure(&device, &config);
        }

        self.redraw();
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

impl Renderer {
    pub async fn new(renderer_type: RendererType, renderer_options: &RendererOptions) -> Self {
        match renderer_type {
            RendererType::Window(window) => {
                let width = window.get_width();
                let height = window.get_height();

                let instance = Instance::default();
                let surface = instance
                    .create_surface(window.get_handle())
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
                            label: None,
                            required_features: Features::empty(),
                            required_limits: Limits::downlevel_webgl2_defaults()
                                .using_resolution(adapter.limits()),
                        },
                        None,
                    )
                    .await
                    .expect("Failed to find device");
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Wgsl(Cow::Borrowed(
                        "@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
",
                    )),
                });

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

                let swapchain_capabilities = surface.get_capabilities(&adapter);
                let swapchain_format = swapchain_capabilities.formats[0];

                let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(swapchain_format.into())],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

                if let Some(config) = surface.get_default_config(&adapter, width, height) {
                    surface.configure(&device, &config);
                }

                let rendering_target = RenderingTarget {
                    surface,
                    adapter,
                    device,
                    queue,
                    render_pipeline,
                };

                Self {
                    renderer_options: renderer_options.clone(),

                    rendering_size: RenderingSize::new(width, height),
                    rendering_target,
                }
            }
        }
    }

    pub fn get_width(&self) -> u32 {
        self.rendering_size.width
    }

    pub fn get_height(&self) -> u32 {
        self.rendering_size.height
    }
}

struct RenderingSize {
    pub width: u32,
    pub height: u32,
}

impl RenderingSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

struct RenderingTarget {
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}
