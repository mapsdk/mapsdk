use std::time::Instant;

use dashmap::DashMap;
use geo::Coord;
use glam::{Quat, Vec3};
use wgpu::*;

use crate::{
    map::context::MapState,
    render::{
        camera::Camera,
        draw::{vector_tile::VectorTileDrawable, DrawItem},
        resources::{bind_group::*, pipeline::*, texture::create_depth_texture},
        targets::Window,
    },
    utils::size::PixelSize,
    Canvas,
};

pub mod camera;
pub mod draw;
pub mod resources;
pub mod targets;
pub mod tessellation;

pub struct MapRenderer {
    renderer_options: MapRendererOptions,

    rendering_size: PixelSize,
    rendering_context: MapRenderingContext,

    camera: Camera,
    layer_draw_items: DashMap<String, DashMap<String, DrawItem>>,
}

impl MapRenderer {
    pub async fn new(canvas: Canvas, renderer_options: &MapRendererOptions) -> Self {
        match canvas {
            Canvas::Window(window) => {
                let width = window.width();
                let height = window.height();
                let pixel_ratio = window.scale_factor();

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

                let surface_capabilities = surface.get_capabilities(&adapter);
                let color_target_state = ColorTargetState {
                    format: surface_capabilities.formats[0],
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                };

                let color_sampler = device.create_sampler(&SamplerDescriptor {
                    address_mode_u: AddressMode::ClampToEdge,
                    address_mode_v: AddressMode::ClampToEdge,
                    address_mode_w: AddressMode::ClampToEdge,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Nearest,
                    mipmap_filter: FilterMode::Nearest,
                    ..Default::default()
                });

                let depth_texture = create_depth_texture(&device, width, height);
                let depth_texture_view =
                    depth_texture.create_view(&TextureViewDescriptor::default());

                let image_pipeline = create_image_pipeline(&device, &color_target_state);
                let shape_fill_pipeline = create_shape_fill_pipeline(&device, &color_target_state);
                let shape_stroke_pipeline =
                    create_shape_stroke_pipeline(&device, &color_target_state);
                let symbol_circle_pipeline =
                    create_symbol_circle_pipeline(&device, &color_target_state);

                let rendering_context = MapRenderingContext {
                    pixel_ratio,
                    surface,
                    adapter,
                    device,
                    queue,

                    color_target_state,
                    color_sampler,
                    depth_texture_view,

                    image_pipeline,
                    shape_fill_pipeline,
                    shape_stroke_pipeline,
                    symbol_circle_pipeline,
                };

                let mut camera = Camera::default();
                camera.set_eye(Vec3::new(0.0, 0.0, height as f32 / 2.0));
                camera.set_aspect(width as f32 / height as f32);

                Self {
                    renderer_options: renderer_options.clone(),

                    rendering_size: PixelSize::new(width, height),
                    rendering_context,

                    camera,
                    layer_draw_items: DashMap::new(),
                }
            }
        }
    }

    pub fn add_layer_draw_item(
        &mut self,
        layer_name: &str,
        item_id: &impl ToString,
        draw_item: DrawItem,
    ) {
        self.layer_draw_items
            .entry(layer_name.to_string())
            .or_insert(DashMap::new());

        if let Some(layer) = self.layer_draw_items.get_mut(layer_name) {
            layer.insert(item_id.to_string(), draw_item);
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn contains_layer_draw_item(&self, layer_name: &str, item_id: &impl ToString) -> bool {
        if let Some(layer) = self.layer_draw_items.get(layer_name) {
            layer.contains_key(&item_id.to_string())
        } else {
            false
        }
    }

    pub fn width(&self) -> u32 {
        self.rendering_size.width
    }

    pub fn height(&self) -> u32 {
        self.rendering_size.height
    }

    pub fn remove_layer_draw_item(&mut self, layer_name: &str, item_id: &impl ToString) {
        if let Some(layer) = self.layer_draw_items.get_mut(layer_name) {
            layer.remove(&item_id.to_string());
        }
    }

    pub fn render(&mut self, map_state: &MapState, inter_renderers: &InterRenderers) {
        let instant = Instant::now();

        let MapRenderingContext {
            surface,
            device,
            queue,
            ..
        } = &self.rendering_context;

        if let Ok(surface_texture) = surface.get_current_texture() {
            let surface_view = surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default());
            let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Map CommandEncoder"),
            });
            {
                let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Map RenderPass"),
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

                for layer_name in &map_state.layers_order {
                    if let Some(layer_pair) = self.layer_draw_items.get_mut(layer_name) {
                        layer_pair.value().iter_mut().for_each(|mut draw_item| {
                            draw_item.draw(map_state, &self, inter_renderers, &mut render_pass);
                        });
                    }
                }
            }

            queue.submit(Some(command_encoder.finish()));
            surface_texture.present();
        }

        log::info!("MapRenderer::render elapsed: {:?}", instant.elapsed());
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        map_state: &MapState,
        inter_renderers: &InterRenderers,
    ) {
        self.rendering_size.width = width;
        self.rendering_size.height = height;

        self.update_camera_size();
        self.update_camera_position(map_state.pitch, map_state.yaw);

        let MapRenderingContext {
            surface,
            adapter,
            device,
            ..
        } = &self.rendering_context;

        if let Some(config) = surface.get_default_config(&adapter, width, height) {
            surface.configure(&device, &config);
        }

        let depth_texture = create_depth_texture(&device, width, height);
        self.rendering_context.depth_texture_view =
            depth_texture.create_view(&TextureViewDescriptor::default());

        self.render(&map_state, inter_renderers);
    }

    pub fn set_pitch_yaw(
        &mut self,
        pitch: f64,
        yaw: f64,
        map_state: &MapState,
        inter_renderers: &InterRenderers,
    ) {
        self.update_camera_position(pitch, yaw);
        self.render(&map_state, inter_renderers);
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
pub struct MapRendererOptions {
    background_color: Color,
}

impl Default for MapRendererOptions {
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

impl MapRendererOptions {
    pub fn with_background_color(mut self, v: Color) -> Self {
        self.background_color = v;
        self
    }
}

pub enum MapRendererType {
    Window(Window),
}

pub struct MapRenderingContext {
    pixel_ratio: f64,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    color_target_state: ColorTargetState,
    color_sampler: Sampler,
    depth_texture_view: TextureView,

    image_pipeline: RenderPipeline,
    shape_fill_pipeline: RenderPipeline,
    shape_stroke_pipeline: RenderPipeline,
    symbol_circle_pipeline: RenderPipeline,
}

pub struct InterRenderers {
    pub vector_tile_renderer: VectorTileRenderer,
}

pub struct VectorTileRenderer {
    // rendering_context: VectorTileRenderingContext,
    camera: Camera,
    map_state: MapState,
}

impl VectorTileRenderer {
    pub async fn new() -> Self {
        let instance = Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .expect("Failed to find adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
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

        let color_target_state = ColorTargetState {
            format: TextureFormat::Rgba8UnormSrgb,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        };

        let shape_fill_pipeline = create_shape_fill_pipeline(&device, &color_target_state);
        let shape_stroke_pipeline = create_shape_stroke_pipeline(&device, &color_target_state);
        let symbol_circle_pipeline = create_symbol_circle_pipeline(&device, &color_target_state);

        let rendering_context = VectorTileRenderingContext {
            device,
            queue,

            shape_fill_pipeline,
            shape_stroke_pipeline,
            symbol_circle_pipeline,
        };

        let mut camera = Camera::default();
        camera.set_eye(Vec3::new(2048.0, 2048.0, 2048.0));
        camera.set_up(-Vec3::Y);

        let mut map_state = MapState::default();
        map_state.center = Coord {
            x: 2048.0,
            y: 2048.0,
        };

        Self {
            // rendering_context,
            camera,
            map_state,
        }
    }

    pub fn render(
        &self,
        map_renderer: &MapRenderer,
        vector_tile_drawable: &mut VectorTileDrawable,
    ) {
        let instant = Instant::now();

        let MapRenderingContext {
            device,
            queue,

            shape_fill_pipeline,
            shape_stroke_pipeline,
            symbol_circle_pipeline,
            ..
        } = &map_renderer.rendering_context;

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Vector Tile CommandEncoder"),
        });
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Vector Tile RenderPass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &vector_tile_drawable.texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let map_view_bgl = create_map_view_bgl(device);
            let map_view_bg =
                create_map_view_bg(device, &map_view_bgl, &self.camera, &self.map_state);

            let symbol_circle_params_bgl = create_symbol_circle_params_bgl(device);
            let shape_fill_params_bgl = create_shape_fill_params_bgl(device);
            let shape_stroke_params_bgl = create_shape_stroke_params_bgl(device);

            // draw fills
            {
                render_pass.set_vertex_buffer(0, vector_tile_drawable.fill_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    vector_tile_drawable.fill_index_buffer.slice(..),
                    IndexFormat::Uint16,
                );

                {
                    render_pass.set_pipeline(&shape_fill_pipeline);

                    for (shape_styles_index, (_, shape_styles)) in
                        vector_tile_drawable.layers_shape_styles.iter().enumerate()
                    {
                        if shape_styles.fill_enabled {
                            let shape_fill_params_bg = create_shape_fill_params_bg(
                                device,
                                &shape_fill_params_bgl,
                                vector_tile_drawable.z,
                                &shape_styles,
                            );

                            render_pass.set_bind_group(0, &map_view_bg, &[]);
                            render_pass.set_bind_group(1, &shape_fill_params_bg, &[]);

                            for feature_meta in &vector_tile_drawable.feature_metas {
                                if !feature_meta.shape_is_points
                                    && !feature_meta.shape_is_lines
                                    && feature_meta.shape_styles_index == shape_styles_index
                                {
                                    for index in &feature_meta.fill_buffer_index {
                                        render_pass.draw_indexed(index.0..index.1, index.2, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    render_pass.set_pipeline(&symbol_circle_pipeline);

                    for (shape_styles_index, (_, shape_styles)) in
                        vector_tile_drawable.layers_shape_styles.iter().enumerate()
                    {
                        if shape_styles.fill_enabled {
                            let symbol_circle_params_bg = create_symbol_circle_params_bg(
                                device,
                                1.0,
                                &symbol_circle_params_bgl,
                                vector_tile_drawable.z,
                                &shape_styles,
                            );

                            render_pass.set_bind_group(0, &map_view_bg, &[]);
                            render_pass.set_bind_group(1, &symbol_circle_params_bg, &[]);

                            for feature_meta in &vector_tile_drawable.feature_metas {
                                if feature_meta.shape_is_points
                                    && feature_meta.shape_styles_index == shape_styles_index
                                {
                                    for index in &feature_meta.fill_buffer_index {
                                        render_pass.draw_indexed(index.0..index.1, index.2, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // draw strokes
            {
                render_pass
                    .set_vertex_buffer(0, vector_tile_drawable.stroke_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    vector_tile_drawable.stroke_index_buffer.slice(..),
                    IndexFormat::Uint16,
                );

                {
                    render_pass.set_pipeline(&shape_stroke_pipeline);

                    for (shape_styles_index, (_, shape_styles)) in
                        vector_tile_drawable.layers_shape_styles.iter().enumerate()
                    {
                        if shape_styles.stroke_enabled {
                            {
                                let shape_stroke_params_bg = create_shape_stroke_params_bg(
                                    device,
                                    1.0,
                                    &shape_stroke_params_bgl,
                                    vector_tile_drawable.z,
                                    0,
                                    &shape_styles,
                                );

                                render_pass.set_bind_group(0, &map_view_bg, &[]);
                                render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);

                                for feature_meta in &vector_tile_drawable.feature_metas {
                                    if feature_meta.shape_is_lines
                                        && feature_meta.shape_styles_index == shape_styles_index
                                    {
                                        for index in &feature_meta.stroke_buffer_index {
                                            render_pass.draw_indexed(
                                                index.0..index.1,
                                                index.2,
                                                0..1,
                                            );
                                        }
                                    }
                                }
                            }

                            {
                                let shape_stroke_params_bg = create_shape_stroke_params_bg(
                                    device,
                                    1.0,
                                    &shape_stroke_params_bgl,
                                    vector_tile_drawable.z,
                                    1,
                                    &shape_styles,
                                );

                                render_pass.set_bind_group(0, &map_view_bg, &[]);
                                render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);

                                for feature_meta in &vector_tile_drawable.feature_metas {
                                    if feature_meta.shape_styles_index == shape_styles_index {
                                        for index in &feature_meta.stroke_buffer_index {
                                            render_pass.draw_indexed(
                                                index.0..index.1,
                                                index.2,
                                                0..1,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        queue.submit(Some(command_encoder.finish()));

        log::info!(
            "VectorTileRenderer::render elapsed: {:?}",
            instant.elapsed()
        );
    }
}

pub struct VectorTileRenderingContext {
    device: Device,
    queue: Queue,

    shape_fill_pipeline: RenderPipeline,
    shape_stroke_pipeline: RenderPipeline,
    symbol_circle_pipeline: RenderPipeline,
}
