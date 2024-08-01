use wgpu::*;

use crate::{
    feature::{Feature, Shape},
    render::{
        draw::Drawable,
        resources::{
            bind_group::{create_map_view_bg, create_map_view_bgl},
            buffer::{
                create_index_buffer_from_u16_slice, create_vertex_buffer_from_vec3_f32_slice,
            },
        },
        tessellation::circle::tessellate_circle,
        DrawItem, MapState, Renderer,
    },
};

pub struct FeatureDrawable {
    vertex_buffer: Buffer,
    vertex_index_buffer: Buffer,
    vertex_index_count: u32,
}

impl FeatureDrawable {
    pub fn new(renderer: &Renderer, feature: &Feature, z: f64) -> Self {
        let rendering_context = &renderer.rendering_context;

        let t = match feature.shape() {
            Shape::Circle { center, radius } => {
                tessellate_circle(center, *radius as f32, z as f32, 6)
            }
            _ => todo!(),
        };

        let vertices = t.vertices;
        let vertex_buffer = create_vertex_buffer_from_vec3_f32_slice(
            rendering_context,
            "Feature vertex buffer",
            &vertices,
        );

        let vertex_indices = t.indices;
        let vertex_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Feature vertex index buffer",
            &vertex_indices,
        );

        Self {
            vertex_buffer,
            vertex_index_buffer,
            vertex_index_count: vertex_indices.len() as u32,
        }
    }
}

impl Drawable for FeatureDrawable {
    fn draw(&self, map_state: &MapState, renderer: &Renderer, render_pass: &mut RenderPass) {
        let rendering_context = &renderer.rendering_context;
        let rendering_resources = &renderer.rendering_resources;

        let map_view_bind_group_layout = create_map_view_bgl(rendering_context);
        let map_view_bind_group = create_map_view_bg(
            rendering_context,
            &map_view_bind_group_layout,
            &renderer.camera,
            &map_state,
        );

        render_pass.set_pipeline(&rendering_resources.shape_fill_pipeline);
        render_pass.set_bind_group(0, &map_view_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.vertex_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.vertex_index_count, 0, 0..1);
    }
}

impl Into<DrawItem> for FeatureDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Feature(self)
    }
}
