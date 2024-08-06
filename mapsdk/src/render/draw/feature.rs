use wgpu::*;

use crate::{
    feature::{style::ShapeStyles, Feature, Shape},
    render::{
        create_symbol_circle_params_bg, create_symbol_circle_params_bgl,
        draw::Drawable,
        resources::{
            bind_group::{
                create_map_view_bg, create_map_view_bgl, create_shape_fill_params_bg,
                create_shape_fill_params_bgl, create_shape_stroke_params_bg,
                create_shape_stroke_params_bgl,
            },
            buffer::VertexIndexBuffer,
        },
        tessellation::{circle::tessellate_circle, geometry::tessellate_geometry, Tessellations},
        DrawItem, MapState, Renderer,
    },
};

pub struct FeatureDrawable {
    feature: Feature,
    z: f32,
    shape_styles: ShapeStyles,

    tessellations: Tessellations,

    fill_buffers: Vec<VertexIndexBuffer>,
    stroke_buffers: Vec<VertexIndexBuffer>,
}

impl FeatureDrawable {
    pub fn new(renderer: &Renderer, feature: &Feature, z: f64, shape_styles: &ShapeStyles) -> Self {
        let rendering_context = &renderer.rendering_context;

        let tessellations = match feature.shape() {
            Shape::Circle { center, radius } => tessellate_circle(center, *radius as f32, 6),
            Shape::Geometry(geom) => tessellate_geometry(&geom),
        };

        let mut fill_buffers: Vec<VertexIndexBuffer> = Vec::new();
        let mut stroke_buffers: Vec<VertexIndexBuffer> = Vec::new();

        for fill_vertex_index in &tessellations.fills {
            let buffers =
                VertexIndexBuffer::from_fill_vertex_index(&rendering_context, &fill_vertex_index);
            fill_buffers.push(buffers);
        }

        for stroke_vertex_index in &tessellations.strokes {
            let buffers = VertexIndexBuffer::from_stroke_vertex_index(
                &rendering_context,
                &stroke_vertex_index,
            );
            stroke_buffers.push(buffers);
        }

        Self {
            feature: feature.clone(),
            z: z as f32,
            shape_styles: shape_styles.clone(),

            tessellations,

            fill_buffers,
            stroke_buffers,
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

        if self.feature.shape().is_points() {
            let symbol_circle_params_bgl = create_symbol_circle_params_bgl(rendering_context);
            let symbol_circle_params_bg = create_symbol_circle_params_bg(
                rendering_context,
                &symbol_circle_params_bgl,
                self.z,
                &self.shape_styles,
            );

            for fill_buffer in &self.fill_buffers {
                render_pass.set_pipeline(&rendering_resources.symbol_circle_pipeline);
                render_pass.set_bind_group(0, &map_view_bg, &[]);
                render_pass.set_bind_group(1, &symbol_circle_params_bg, &[]);
                render_pass.set_vertex_buffer(0, fill_buffer.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(fill_buffer.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..fill_buffer.index_count, 0, 0..1);
            }
        } else {
            if self.shape_styles.fill_enabled {
                let shape_fill_params_bgl = create_shape_fill_params_bgl(rendering_context);
                let shape_fill_params_bg = create_shape_fill_params_bg(
                    rendering_context,
                    &shape_fill_params_bgl,
                    self.z,
                    &self.shape_styles,
                );

                for fill_buffer in &self.fill_buffers {
                    render_pass.set_pipeline(&rendering_resources.shape_fill_pipeline);
                    render_pass.set_bind_group(0, &map_view_bg, &[]);
                    render_pass.set_bind_group(1, &shape_fill_params_bg, &[]);
                    render_pass.set_vertex_buffer(0, fill_buffer.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(fill_buffer.index_buffer.slice(..), IndexFormat::Uint16);
                    render_pass.draw_indexed(0..fill_buffer.index_count, 0, 0..1);
                }
            }

            if self.shape_styles.stroke_enabled {
                let shape_stroke_params_bgl = create_shape_stroke_params_bgl(rendering_context);
                let shape_stroke_params_bg = create_shape_stroke_params_bg(
                    rendering_context,
                    &shape_stroke_params_bgl,
                    self.z,
                    &self.shape_styles,
                );

                for stroke_buffer in &self.stroke_buffers {
                    render_pass.set_pipeline(&rendering_resources.shape_stroke_pipeline);
                    render_pass.set_bind_group(0, &map_view_bg, &[]);
                    render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);
                    render_pass.set_vertex_buffer(0, stroke_buffer.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        stroke_buffer.index_buffer.slice(..),
                        IndexFormat::Uint16,
                    );
                    render_pass.draw_indexed(0..stroke_buffer.index_count, 0, 0..1);
                }
            }
        }
    }
}

impl Into<DrawItem> for FeatureDrawable {
    fn into(self) -> DrawItem {
        DrawItem::Feature(self)
    }
}
