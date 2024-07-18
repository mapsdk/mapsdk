use image::DynamicImage;
use wgpu::*;

use crate::render::{
    create_image_params_bgl, create_image_texture_bgl, create_texture_from_image,
    draw::Drawable,
    resources::{
        buffer::{
            create_index_buffer_from_u16_slice, create_uniform_buffer_from_f32_slice,
            create_uniform_buffer_from_vec4_f32_slice, create_vertex_buffer_from_vec2_f32_slice,
        },
        layout::create_camera_bgl,
    },
    DrawItem, ImageCoords, MapState, Renderer,
};

pub struct ImageDrawable {
    texture: Texture,

    vertex_buffer: Buffer,
    vertex_index_buffer: Buffer,
}

impl ImageDrawable {
    pub fn new(renderer: &Renderer, image: &DynamicImage, coords: &ImageCoords) -> Self {
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

        let vertices = [
            [coords.lt.x() as f32, coords.lt.y() as f32],
            [coords.lb.x() as f32, coords.lb.y() as f32],
            [coords.rt.x() as f32, coords.rt.y() as f32],
            [coords.rb.x() as f32, coords.rb.y() as f32],
        ];

        let vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            rendering_context,
            "Image vertex buffer",
            &vertices,
        );

        let vertex_indices: [u16; 6] = [0, 1, 2, 2, 1, 3];
        let vertex_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Image vertex index buffer",
            &vertex_indices,
        );

        Self {
            texture,

            vertex_buffer,
            vertex_index_buffer,
        }
    }
}

impl Drawable for ImageDrawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        let rendering_context = &renderer.rendering_context;
        let rendering_resources = &renderer.rendering_resources;

        let camera_buffer = create_uniform_buffer_from_vec4_f32_slice(
            rendering_context,
            "Camera buffer",
            &renderer.camera.view_proj(),
        );
        let camera_bind_group_layout = create_camera_bgl(rendering_context);
        let camera_bind_group = rendering_context
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let texture = &self.texture;
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let texture_bind_group_layout = create_image_texture_bgl(rendering_context);
        let texture_bind_group = rendering_context
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Image texture buffer"),
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&rendering_resources.image_sampler),
                    },
                ],
            });

        let map_center_buffer = create_uniform_buffer_from_f32_slice(
            rendering_context,
            "Image map center buffer",
            &[map_state.center().x() as f32, map_state.center().y() as f32],
        );

        let map_res_buffer = create_uniform_buffer_from_f32_slice(
            rendering_context,
            "Image map res buffer",
            &[(map_state.zoom_res() * map_state.map_res_ratio()) as f32],
        );

        let params_bind_group_layout = create_image_params_bgl(rendering_context);
        let params_bind_group = rendering_context
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Image params bind group"),
                layout: &params_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: map_center_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: map_res_buffer.as_entire_binding(),
                    },
                ],
            });

        render_pass.set_pipeline(&rendering_resources.image_pipeline);
        render_pass.set_bind_group(0, &camera_bind_group, &[]);
        render_pass.set_bind_group(1, &texture_bind_group, &[]);
        render_pass.set_bind_group(2, &params_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.vertex_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}

impl Into<DrawItem> for ImageDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Image(self)
    }
}
