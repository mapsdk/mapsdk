use wgpu::*;

use crate::{
    feature::{style::ShapeStyles, Shape},
    render::{
        create_image_params_bg, create_image_params_bgl, create_image_texture_bg,
        create_image_texture_bgl, create_map_view_bg, create_map_view_bgl,
        draw::Drawable,
        resources::{
            buffer::{
                create_index_buffer_from_u16_slice, create_vertex_buffer_from_vec2_f32_slice,
                create_vertex_buffer_from_vec7_f32_slice,
            },
            texture::create_texture,
        },
        tessellation::{circle::tessellate_circle, geometry::tessellate_geometry},
        DrawItem, InterRenderers, MapRenderer, MapRenderingContext, MapState,
    },
    vector_tile::VectorTile,
};

pub struct VectorTileDrawable {
    pub z: f32,
    pub layers_shape_styles: Vec<(String, ShapeStyles)>,

    pub fill_vertex_buffer: Buffer,
    pub fill_index_buffer: Buffer,
    pub stroke_vertex_buffer: Buffer,
    pub stroke_index_buffer: Buffer,
    pub feature_metas: Vec<VectorTileFeatureMeta>,

    pub texture_view: TextureView,
    pub texture_updated_zoom_res: f64,
    pub texture_vertex_buffer: Buffer,
    pub texture_index_buffer: Buffer,
}

impl VectorTileDrawable {
    pub fn new(
        vector_tile: &VectorTile,
        z: f64,
        layers_shape_styles: &Vec<(String, ShapeStyles)>,
        map_renderer: &MapRenderer,
        inter_renderers: &InterRenderers,
    ) -> Self {
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
            if let Some(layer) = &vector_tile.layers().get(layer_name) {
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

        let device = &map_renderer.rendering_context.device;

        let fill_vertex_buffer =
            create_vertex_buffer_from_vec2_f32_slice(&device, "Fill Vertex Buffer", &fill_vertices);
        let fill_index_buffer =
            create_index_buffer_from_u16_slice(&device, "Fill Index Buffer", &fill_indices);

        let stroke_vertex_buffer = create_vertex_buffer_from_vec7_f32_slice(
            &device,
            "Stroke Vertex Buffer",
            &stroke_vertices,
        );
        let stroke_index_buffer =
            create_index_buffer_from_u16_slice(&device, "Stroke Index Buffer", &stroke_indices);

        let texture = create_texture(device, 4096);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let bbox = vector_tile.bbox();
        let texture_vertices = [
            [bbox.min().x as f32, bbox.max().y as f32],
            [bbox.max().x as f32, bbox.max().y as f32],
            [bbox.min().x as f32, bbox.min().y as f32],
            [bbox.max().x as f32, bbox.min().y as f32],
        ];
        let texture_vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            &device,
            "Vector Tile Texture VertexBuffer",
            &texture_vertices,
        );

        let texture_indices: [u16; 4] = [0, 1, 2, 3];
        let texture_index_buffer = create_index_buffer_from_u16_slice(
            &device,
            "Vector Tile Texture IndexBuffer",
            &texture_indices,
        );

        Self {
            z: z as f32,
            layers_shape_styles: layers_shape_styles.clone(),

            fill_vertex_buffer,
            fill_index_buffer,
            stroke_vertex_buffer,
            stroke_index_buffer,
            feature_metas,

            texture_view,
            texture_updated_zoom_res: 0.0,
            texture_vertex_buffer,
            texture_index_buffer,
        }
    }
}

impl Drawable for VectorTileDrawable {
    fn draw(
        &mut self,
        map_state: &MapState,
        map_renderer: &MapRenderer,
        inter_renderers: &InterRenderers,
        render_pass: &mut RenderPass,
    ) {
        // Update texture if needed
        if self.texture_updated_zoom_res != map_state.zoom_res {
            inter_renderers
                .vector_tile_renderer
                .render(map_renderer, self);
            self.texture_updated_zoom_res = map_state.zoom_res;
        }

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

impl Into<DrawItem> for VectorTileDrawable {
    fn into(self) -> DrawItem {
        DrawItem::VectorTile(self)
    }
}

pub struct VectorTileFeatureMeta {
    pub shape_is_points: bool,
    pub shape_is_lines: bool,
    pub shape_styles_index: usize,

    pub fill_buffer_index: Vec<(u32, u32, i32)>,
    pub stroke_buffer_index: Vec<(u32, u32, i32)>,
}
