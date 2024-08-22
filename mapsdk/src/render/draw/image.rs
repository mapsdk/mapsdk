use geo::Rect;
use image::DynamicImage;
use wgpu::*;

use crate::render::{
    draw::Drawable,
    resources::{
        bind_group::{
            create_image_params_bg, create_image_params_bgl, create_image_texture_bg,
            create_image_texture_bgl, create_map_view_bg, create_map_view_bgl,
        },
        buffer::{create_index_buffer_from_u16_slice, create_vertex_buffer_from_vec2_f32_slice},
        texture::create_texture_from_image,
    },
    DrawItem, InterRenderers, MapRenderer, MapRenderingContext, MapState,
};

pub struct ImageDrawable {
    z: f32,

    texture_view: TextureView,
    texture_vertex_buffer: Buffer,
    texture_index_buffer: Buffer,
}

impl ImageDrawable {
    pub fn new(map_renderer: &MapRenderer, image: &DynamicImage, bbox: &Rect, z: f64) -> Self {
        let MapRenderingContext { device, queue, .. } = &map_renderer.rendering_context;

        let texture = create_texture_from_image(&device, image);
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image.to_rgba8(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            texture.size(),
        );
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let texture_vertices = [
            [bbox.min().x as f32, bbox.max().y as f32],
            [bbox.max().x as f32, bbox.max().y as f32],
            [bbox.min().x as f32, bbox.min().y as f32],
            [bbox.max().x as f32, bbox.min().y as f32],
        ];
        let texture_vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            &device,
            "Image Texture VertexBuffer",
            &texture_vertices,
        );

        let texture_indices: [u16; 4] = [0, 1, 2, 3];
        let texture_index_buffer = create_index_buffer_from_u16_slice(
            &device,
            "Image Texture IndexBuffer",
            &texture_indices,
        );

        Self {
            z: z as f32,

            texture_view,
            texture_vertex_buffer,
            texture_index_buffer,
        }
    }
}

impl Drawable for ImageDrawable {
    fn draw(
        &mut self,
        map_state: &MapState,
        map_renderer: &MapRenderer,
        _inter_renderers: &InterRenderers,
        render_pass: &mut RenderPass,
    ) {
        let MapRenderingContext {
            device,
            color_sampler,
            image_pipeline,
            ..
        } = &map_renderer.rendering_context;

        let map_view_bgl = create_map_view_bgl(device);
        let map_view_bg =
            create_map_view_bg(device, &map_view_bgl, &map_renderer.camera, &map_state);

        let image_texture_bgl = create_image_texture_bgl(device);
        let image_texture_bg = create_image_texture_bg(
            device,
            &image_texture_bgl,
            &self.texture_view,
            &color_sampler,
        );

        let image_params_bgl = create_image_params_bgl(device);
        let image_params_bg = create_image_params_bg(device, &image_params_bgl, self.z);

        render_pass.set_pipeline(&image_pipeline);
        render_pass.set_bind_group(0, &map_view_bg, &[]);
        render_pass.set_bind_group(1, &image_texture_bg, &[]);
        render_pass.set_bind_group(2, &image_params_bg, &[]);
        render_pass.set_vertex_buffer(0, self.texture_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.texture_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..4, 0, 0..1);
    }
}

impl Into<DrawItem> for ImageDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Image(self)
    }
}
