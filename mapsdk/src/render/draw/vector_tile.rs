use wgpu::*;

use crate::{
    feature::style::ShapeStyles,
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
        tessellation::vector_tile::{VectorTileShapeMeta, VectorTileTessellation},
        DrawItem, InterRenderers, MapOptions, MapRenderer, MapRenderingContext, MapState,
    },
    tiling::TileId,
};

pub struct VectorTileDrawable {
    pub tile_id: TileId,
    pub z: f32,
    pub layers_shape_styles: Vec<(String, ShapeStyles)>,

    pub fill_vertex_buffer: Buffer,
    pub fill_index_buffer: Buffer,
    pub stroke_vertex_buffer: Buffer,
    pub stroke_index_buffer: Buffer,
    pub shape_metas: Vec<VectorTileShapeMeta>,

    pub texture_view: TextureView,
    pub texture_updated_zoom_res: f64,
    pub texture_vertex_buffer: Buffer,
    pub texture_index_buffer: Buffer,
}

impl VectorTileDrawable {
    pub fn new(
        tile_id: &TileId,
        vector_tile_tessellation: &VectorTileTessellation,
        z: f64,
        layers_shape_styles: &Vec<(String, ShapeStyles)>,
        map_renderer: &MapRenderer,
        _inter_renderers: &InterRenderers,
    ) -> Self {
        let VectorTileTessellation {
            fill_vertices,
            fill_indices,
            stroke_vertices,
            stroke_indices,

            tile_bbox,
            shape_metas,
        } = vector_tile_tessellation;

        let MapRenderingContext {
            device,
            color_target_state,
            ..
        } = &map_renderer.rendering_context;

        let fill_vertex_buffer =
            create_vertex_buffer_from_vec2_f32_slice(device, "Fill Vertex Buffer", &fill_vertices);
        let fill_index_buffer =
            create_index_buffer_from_u16_slice(device, "Fill Index Buffer", &fill_indices);

        let stroke_vertex_buffer = create_vertex_buffer_from_vec7_f32_slice(
            device,
            "Stroke Vertex Buffer",
            &stroke_vertices,
        );
        let stroke_index_buffer =
            create_index_buffer_from_u16_slice(device, "Stroke Index Buffer", &stroke_indices);

        let texture = create_texture(device, 4096, 4096, color_target_state.format);
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let texture_vertices = [
            [tile_bbox[0], tile_bbox[3]],
            [tile_bbox[2], tile_bbox[3]],
            [tile_bbox[0], tile_bbox[1]],
            [tile_bbox[2], tile_bbox[1]],
        ];
        let texture_vertex_buffer = create_vertex_buffer_from_vec2_f32_slice(
            device,
            "Vector Tile Texture VertexBuffer",
            &texture_vertices,
        );

        let texture_indices: [u16; 4] = [0, 1, 2, 3];
        let texture_index_buffer = create_index_buffer_from_u16_slice(
            device,
            "Vector Tile Texture IndexBuffer",
            &texture_indices,
        );

        Self {
            tile_id: tile_id.clone(),
            z: z as f32,
            layers_shape_styles: layers_shape_styles.clone(),

            fill_vertex_buffer,
            fill_index_buffer,
            stroke_vertex_buffer,
            stroke_index_buffer,
            shape_metas: shape_metas.clone(),

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
        map_options: &MapOptions,
        map_state: &MapState,
        map_renderer: &MapRenderer,
        inter_renderers: &InterRenderers,
        render_pass: &mut RenderPass,
    ) {
        // Update texture if needed
        if self.texture_updated_zoom_res != map_state.zoom_res {
            inter_renderers
                .vector_tile_renderer
                .render(map_options, map_state, map_renderer, self);
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
