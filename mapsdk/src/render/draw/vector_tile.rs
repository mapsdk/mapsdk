use wgpu::*;

use crate::{
    feature::{style::ShapeStyles, Shape},
    render::{
        create_map_view_bg, create_map_view_bgl, create_shape_fill_params_bg,
        create_shape_fill_params_bgl, create_shape_stroke_params_bg,
        create_shape_stroke_params_bgl, create_symbol_circle_params_bg,
        create_symbol_circle_params_bgl,
        draw::Drawable,
        resources::buffer::{
            create_index_buffer_from_u16_slice, create_vertex_buffer_from_vec2_f32_slice,
            create_vertex_buffer_from_vec7_f32_slice,
        },
        tessellation::{circle::tessellate_circle, geometry::tessellate_geometry},
        DrawItem, MapState, Renderer,
    },
    vector_tile::VectorTile,
};

pub struct VectorTileDrawable {
    layers_shape_styles: Vec<(String, ShapeStyles)>,
    z: f32,

    fill_vertex_buffer: Buffer,
    fill_index_buffer: Buffer,
    stroke_vertex_buffer: Buffer,
    stroke_index_buffer: Buffer,
    feature_metas: Vec<VectorTileFeatureMeta>,
}

impl VectorTileDrawable {
    pub fn new(
        renderer: &Renderer,
        vector_tile: &VectorTile,
        z: f64,
        layers_shape_styles: &Vec<(String, ShapeStyles)>,
    ) -> Self {
        let rendering_context = &renderer.rendering_context;

        let mut fill_vertices: Vec<[f32; 2]> = Vec::new();
        let mut fill_indices: Vec<u16> = Vec::new();
        let mut stroke_vertices: Vec<[f32; 7]> = Vec::new();
        let mut stroke_indices: Vec<u16> = Vec::new();
        let mut feature_metas = Vec::new();

        let mut fill_vertex_start: i32 = 0;
        let mut fill_index_start: u32 = 0;
        let mut stroke_vertex_start: i32 = 0;
        let mut stroke_index_start: u32 = 0;

        for (shape_styles_index, (layer_name, _)) in layers_shape_styles.iter().enumerate() {
            if let Some(layer) = &vector_tile.layers.get(layer_name) {
                for feature in &layer.features {
                    let shape = feature.shape();

                    let tessellations = match shape {
                        Shape::Circle { center, radius } => tessellate_circle(center, *radius, 6),
                        Shape::Geometry(geom) => tessellate_geometry(geom),
                    };

                    let mut fill_buffer_index = Vec::new();
                    let mut stroke_buffer_index = Vec::new();

                    for fill_vertex_index in &tessellations.fills {
                        fill_vertices.extend_from_slice(&fill_vertex_index.vertices);
                        fill_indices.extend_from_slice(&fill_vertex_index.indices);

                        let index_count = fill_vertex_index.indices.len() as u32;
                        fill_buffer_index.push((
                            fill_index_start,
                            fill_index_start + index_count,
                            fill_vertex_start,
                        ));

                        fill_vertex_start += fill_vertex_index.vertices.len() as i32;
                        fill_index_start += index_count;
                    }

                    for stroke_vertex_index in &tessellations.strokes {
                        stroke_vertices.extend_from_slice(&stroke_vertex_index.vertices);
                        stroke_indices.extend_from_slice(&stroke_vertex_index.indices);

                        let index_count = stroke_vertex_index.indices.len() as u32;
                        stroke_buffer_index.push((
                            stroke_index_start,
                            stroke_index_start + index_count,
                            stroke_vertex_start,
                        ));

                        stroke_vertex_start += stroke_vertex_index.vertices.len() as i32;
                        stroke_index_start += index_count;
                    }

                    feature_metas.push(VectorTileFeatureMeta {
                        shape_is_points: shape.is_points(),
                        shape_is_lines: shape.is_lines(),
                        shape_styles_index,

                        fill_buffer_index,
                        stroke_buffer_index,
                    });
                }
            }
        }

        let fill_vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            rendering_context,
            "Fill Vertex Buffer",
            &fill_vertices,
        );
        let fill_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Fill Index Buffer",
            &fill_indices,
        );

        let stroke_vertex_buffer = create_vertex_buffer_from_vec7_f32_slice(
            rendering_context,
            "Stroke Vertex Buffer",
            &stroke_vertices,
        );
        let stroke_index_buffer = create_index_buffer_from_u16_slice(
            rendering_context,
            "Stroke Index Buffer",
            &stroke_indices,
        );

        Self {
            layers_shape_styles: layers_shape_styles.clone(),
            z: z as f32,

            fill_vertex_buffer,
            fill_index_buffer,
            stroke_vertex_buffer,
            stroke_index_buffer,
            feature_metas,
        }
    }
}

impl Drawable for VectorTileDrawable {
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

        let symbol_circle_params_bgl = create_symbol_circle_params_bgl(rendering_context);
        let shape_fill_params_bgl = create_shape_fill_params_bgl(rendering_context);
        let shape_stroke_params_bgl = create_shape_stroke_params_bgl(rendering_context);

        // draw fills
        {
            render_pass.set_vertex_buffer(0, self.fill_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.fill_index_buffer.slice(..), IndexFormat::Uint16);

            {
                render_pass.set_pipeline(&rendering_resources.shape_fill_pipeline);

                for (shape_styles_index, (_, shape_styles)) in
                    self.layers_shape_styles.iter().enumerate()
                {
                    if shape_styles.fill_enabled {
                        let shape_fill_params_bg = create_shape_fill_params_bg(
                            rendering_context,
                            &shape_fill_params_bgl,
                            self.z,
                            &shape_styles,
                        );

                        render_pass.set_bind_group(0, &map_view_bg, &[]);
                        render_pass.set_bind_group(1, &shape_fill_params_bg, &[]);

                        for feature_meta in &self.feature_metas {
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
                render_pass.set_pipeline(&rendering_resources.symbol_circle_pipeline);

                for (shape_styles_index, (_, shape_styles)) in
                    self.layers_shape_styles.iter().enumerate()
                {
                    if shape_styles.fill_enabled {
                        let symbol_circle_params_bg = create_symbol_circle_params_bg(
                            rendering_context,
                            &symbol_circle_params_bgl,
                            self.z,
                            &shape_styles,
                        );

                        render_pass.set_bind_group(0, &map_view_bg, &[]);
                        render_pass.set_bind_group(1, &symbol_circle_params_bg, &[]);

                        for feature_meta in &self.feature_metas {
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
            render_pass.set_vertex_buffer(0, self.stroke_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.stroke_index_buffer.slice(..), IndexFormat::Uint16);

            {
                render_pass.set_pipeline(&rendering_resources.shape_stroke_pipeline);

                for (shape_styles_index, (_, shape_styles)) in
                    self.layers_shape_styles.iter().enumerate()
                {
                    if shape_styles.stroke_enabled {
                        {
                            let shape_stroke_params_bg = create_shape_stroke_params_bg(
                                rendering_context,
                                &shape_stroke_params_bgl,
                                self.z,
                                0,
                                &shape_styles,
                            );

                            render_pass.set_bind_group(0, &map_view_bg, &[]);
                            render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);

                            for feature_meta in &self.feature_metas {
                                if feature_meta.shape_is_lines
                                    && feature_meta.shape_styles_index == shape_styles_index
                                {
                                    for index in &feature_meta.stroke_buffer_index {
                                        render_pass.draw_indexed(index.0..index.1, index.2, 0..1);
                                    }
                                }
                            }
                        }

                        {
                            let shape_stroke_params_bg = create_shape_stroke_params_bg(
                                rendering_context,
                                &shape_stroke_params_bgl,
                                self.z,
                                1,
                                &shape_styles,
                            );

                            render_pass.set_bind_group(0, &map_view_bg, &[]);
                            render_pass.set_bind_group(1, &shape_stroke_params_bg, &[]);

                            for feature_meta in &self.feature_metas {
                                if feature_meta.shape_styles_index == shape_styles_index {
                                    for index in &feature_meta.stroke_buffer_index {
                                        render_pass.draw_indexed(index.0..index.1, index.2, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Into<DrawItem> for VectorTileDrawable {
    fn into(self) -> DrawItem {
        DrawItem::VectorTile(self)
    }
}

struct VectorTileFeatureMeta {
    shape_is_points: bool,
    shape_is_lines: bool,
    shape_styles_index: usize,

    fill_buffer_index: Vec<(u32, u32, i32)>,
    stroke_buffer_index: Vec<(u32, u32, i32)>,
}
