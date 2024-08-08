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
    DrawItem, MapState, Renderer,
};

pub struct ImageDrawable {
    texture_view: TextureView,
    z: f32,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl ImageDrawable {
    pub fn new(renderer: &Renderer, image: &DynamicImage, bbox: &Rect, z: f64) -> Self {
        let rendering_context = &renderer.rendering_context;

        let texture = create_texture_from_image(rendering_context, image);
        rendering_context.queue.write_texture(
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

        let vertices = [
            [bbox.min().x as f32, bbox.max().y as f32],
            [bbox.max().x as f32, bbox.max().y as f32],
            [bbox.min().x as f32, bbox.min().y as f32],
            [bbox.max().x as f32, bbox.min().y as f32],
        ];
        let vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            rendering_context,
            "Image VertexBuffer",
            &vertices,
        );

        let indices: [u16; 4] = [0, 1, 2, 3];
        let index_buffer =
            create_index_buffer_from_u16_slice(rendering_context, "Image IndexBuffer", &indices);

        Self {
            z: z as f32,
            texture_view,

            vertex_buffer,
            index_buffer,
        }
    }
}

impl Drawable for ImageDrawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        let rendering_context = &renderer.rendering_context;
        let rendering_resources = &renderer.rendering_resources;

        let map_view_bgl = create_map_view_bgl(rendering_context);
        let map_view_bg = create_map_view_bg(
            rendering_context,
            &map_view_bgl,
            &renderer.camera,
            &map_state,
        );

        let image_texture_bgl = create_image_texture_bgl(rendering_context);
        let image_texture_bg = create_image_texture_bg(
            rendering_context,
            &image_texture_bgl,
            &self.texture_view,
            &rendering_resources.color_sampler,
        );

        let image_params_bgl = create_image_params_bgl(rendering_context);
        let image_params_bg = create_image_params_bg(rendering_context, &image_params_bgl, self.z);

        render_pass.set_pipeline(&rendering_resources.image_pipeline);
        render_pass.set_bind_group(0, &map_view_bg, &[]);
        render_pass.set_bind_group(1, &image_texture_bg, &[]);
        render_pass.set_bind_group(2, &image_params_bg, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..4, 0, 0..1);
    }
}

impl Into<DrawItem> for ImageDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Image(self)
    }
}
