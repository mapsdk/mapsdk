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
        tessellation::{circle::tessellate_circle, geometry::tessellate_geometry},
        DrawItem, InterRenderers, MapOptions, MapRenderer, MapRenderingContext, MapState,
    },
};

pub struct FeatureDrawable {
    feature: Feature,
    z: f32,
    shape_styles: ShapeStyles,

    fill_buffers: Vec<VertexIndexBuffer>,
    stroke_buffers: Vec<VertexIndexBuffer>,
}

impl FeatureDrawable {
    pub fn new(
        map_renderer: &MapRenderer,
        feature: &Feature,
        z: f64,
        shape_styles: &ShapeStyles,
    ) -> Self {
        let MapRenderingContext { device, .. } = &map_renderer.rendering_context;

        let tessellations = match feature.shape() {
            Shape::Circle { center, radius } => tessellate_circle(center, *radius as f32, 6),
            Shape::Geometry(geom) => tessellate_geometry(&geom),
        };

        let mut fill_buffers: Vec<VertexIndexBuffer> = Vec::new();
        let mut stroke_buffers: Vec<VertexIndexBuffer> = Vec::new();

        for fill_vertex_index in &tessellations.fills {
            let buffers = VertexIndexBuffer::from_fill_vertex_index(&device, &fill_vertex_index);
            fill_buffers.push(buffers);
        }

        for stroke_vertex_index in &tessellations.strokes {
            let buffers =
                VertexIndexBuffer::from_stroke_vertex_index(&device, &stroke_vertex_index);
            stroke_buffers.push(buffers);
        }

        Self {
            feature: feature.clone(),
            z: z as f32,
            shape_styles: shape_styles.clone(),

            fill_buffers,
            stroke_buffers,
        }
    }
}

impl Drawable for FeatureDrawable {
    fn draw(
        &mut self,
        _map_options: &MapOptions,
        map_state: &MapState,
        map_renderer: &MapRenderer,
        _inter_renderers: &InterRenderers,
        render_pass: &mut RenderPass,
    ) {
        let MapRenderingContext {
            device,
            pixel_ratio,

            shape_fill_pipeline,
            shape_stroke_pipeline,
            symbol_circle_pipeline,
            ..
        } = &map_renderer.rendering_context;

        let map_view_bgl = create_map_view_bgl(device);
        let map_view_bg =
            create_map_view_bg(device, &map_view_bgl, &map_renderer.camera, &map_state);

        if self.feature.shape().is_points() {
            let symbol_circle_params_bgl = create_symbol_circle_params_bgl(device);
            let symbol_circle_params_bg = create_symbol_circle_params_bg(
                device,
                *pixel_ratio as f32,
                &symbol_circle_params_bgl,
                self.z,
                &self.shape_styles,
            );

            for fill_buffer in &self.fill_buffers {
                render_pass.set_pipeline(&symbol_circle_pipeline);
                render_pass.set_bind_group(0, &map_view_bg, &[]);
                render_pass.set_bind_group(1, &symbol_circle_params_bg, &[]);
                render_pass.set_vertex_buffer(0, fill_buffer.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(fill_buffer.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..fill_buffer.index_count, 0, 0..1);
            }
        } else {
            if self.shape_styles.fill_enabled {
                let shape_fill_params_bgl = create_shape_fill_params_bgl(device);
                let shape_fill_params_bg = create_shape_fill_params_bg(
                    device,
                    &shape_fill_params_bgl,
                    self.z,
                    &self.shape_styles,
                );

                for fill_buffer in &self.fill_buffers {
                    render_pass.set_pipeline(&shape_fill_pipeline);
                    render_pass.set_bind_group(0, &map_view_bg, &[]);
                    render_pass.set_bind_group(1, &shape_fill_params_bg, &[]);
                    render_pass.set_vertex_buffer(0, fill_buffer.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(fill_buffer.index_buffer.slice(..), IndexFormat::Uint16);
                    render_pass.draw_indexed(0..fill_buffer.index_count, 0, 0..1);
                }
            }

            if self.shape_styles.stroke_enabled {
                let align = if self.feature.shape().is_lines() {
                    0
                } else {
                    1
                };

                let shape_stroke_params_bgl = create_shape_stroke_params_bgl(device);
                let shape_stroke_params_bg = create_shape_stroke_params_bg(
                    device,
                    *pixel_ratio as f32,
                    &shape_stroke_params_bgl,
                    self.z,
                    align,
                    &self.shape_styles,
                );

                for stroke_buffer in &self.stroke_buffers {
                    render_pass.set_pipeline(&shape_stroke_pipeline);
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
