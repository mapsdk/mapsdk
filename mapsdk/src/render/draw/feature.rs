use wgpu::*;

use crate::{
    feature::{style::ShapeStyles, Feature, Shape},
    render::{
        draw::Drawable,
        resources::{
            bind_group::{
                create_map_view_bg, create_map_view_bgl, create_shape_fill_params_bg,
                create_shape_fill_params_bgl, create_shape_stroke_params_bg,
                create_shape_stroke_params_bgl,
            },
            buffer::{
                create_index_buffer_from_u16_slice, create_vertex_buffer_from_vec2_f32_slice,
                create_vertex_buffer_from_vec5_f32_slice,
            },
        },
        tessellation::{circle::tessellate_circle, Tessellation},
        DrawItem, MapState, Renderer,
    },
};

pub struct FeatureDrawable {
    feature: Feature,
    z: f32,
    shape_styles: ShapeStyles,

    tessellation: Tessellation,

    fill_vertex_buffer: Buffer,
    fill_index_buffer: Buffer,
    stroke_vertex_buffer: Buffer,
    stroke_index_buffer: Buffer,
}

impl FeatureDrawable {
    pub fn new(renderer: &Renderer, feature: &Feature, z: f64, shape_styles: &ShapeStyles) -> Self {
        let rendering_context = &renderer.rendering_context;

        let tessellation = match feature.shape() {
            Shape::Circle { center, radius } => tessellate_circle(center, *radius as f32, 6),
            _ => todo!(),
        };

        let fill_vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            rendering_context,
            "Feature Fill VertexBuffer",
            &tessellation.fill_vertices,
        );

        let fill_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Feature Fill IndexBuffer",
            &tessellation.fill_indices,
        );

        let stroke_vertex_buffer = create_vertex_buffer_from_vec5_f32_slice(
            rendering_context,
            "Feature Stroke VertexBuffer",
            &tessellation.stroke_vertices,
        );

        let stroke_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Feature Stroke IndexBuffer",
            &tessellation.stroke_indices,
        );

        Self {
            feature: feature.clone(),
            z: z as f32,
            shape_styles: shape_styles.clone(),

            tessellation,

            fill_vertex_buffer,
            fill_index_buffer,
            stroke_vertex_buffer,
            stroke_index_buffer,
        }
    }
}

impl Drawable for FeatureDrawable {
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

        if self.shape_styles.fill_enabled {
            let shape_fill_params_bgl = create_shape_fill_params_bgl(rendering_context);
            let shape_fill_params_bg = create_shape_fill_params_bg(
                rendering_context,
                &shape_fill_params_bgl,
                self.z,
                &self.shape_styles,
            );

            render_pass.set_pipeline(&rendering_resources.shape_fill_pipeline);
            render_pass.set_bind_group(0, &map_view_bg, &[]);
            render_pass.set_bind_group(1, &shape_fill_params_bg, &[]);
            render_pass.set_vertex_buffer(0, self.fill_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.fill_index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.tessellation.fill_indices.len() as u32, 0, 0..1);
        }

        if self.shape_styles.stroke_enabled {
            let shape_stroke_params_bgl = create_shape_stroke_params_bgl(rendering_context);
            let shape_stroke_params_bg = create_shape_stroke_params_bg(
                rendering_context,
                &shape_stroke_params_bgl,
                self.z,
                &self.shape_styles,
            );

            render_pass.set_pipeline(&rendering_resources.shape_stroke_pipeline);
            render_pass.set_bind_group(0, &map_view_bg, &[]);
            render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);
            render_pass.set_vertex_buffer(0, self.stroke_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.stroke_index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.tessellation.stroke_indices.len() as u32, 0, 0..1);
        }
    }
}

impl Into<DrawItem> for FeatureDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Feature(self)
    }
}
