use crate::{
    feature::{style::ShapeStyles, Shape},
    render::tessellation::{circle::tessellate_circle, geometry::tessellate_geometry},
    vector_tile::VectorTile,
};

#[derive(Clone)]
pub struct VectorTileTessellation {
    pub fill_vertices: Vec<[f32; 2]>,
    pub fill_indices: Vec<u16>,
    pub stroke_vertices: Vec<[f32; 7]>,
    pub stroke_indices: Vec<u16>,

    pub tile_bbox: [f32; 4],
    pub shape_metas: Vec<VectorTileShapeMeta>,
}

impl VectorTileTessellation {
    pub fn new(vector_tile: &VectorTile, layers_shape_styles: &Vec<(String, ShapeStyles)>) -> Self {
        let mut fill_vertices = Vec::new();
        let mut fill_indices = Vec::new();
        let mut stroke_vertices = Vec::new();
        let mut stroke_indices = Vec::new();
        let mut shape_metas = Vec::new();

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

                    shape_metas.push(VectorTileShapeMeta {
                        shape_is_points: shape.is_points(),
                        shape_is_lines: shape.is_lines(),
                        shape_styles_index,

                        fill_buffer_index,
                        stroke_buffer_index,
                    });
                }
            }
        }

        let bbox = vector_tile.bbox();
        let tile_bbox = [
            bbox.min().x as f32,
            bbox.min().y as f32,
            bbox.max().x as f32,
            bbox.max().y as f32,
        ];

        Self {
            fill_vertices,
            fill_indices,
            stroke_vertices,
            stroke_indices,

            tile_bbox,
            shape_metas,
        }
    }
}

#[derive(Clone)]
pub struct VectorTileShapeMeta {
    pub shape_is_points: bool,
    pub shape_is_lines: bool,
    pub shape_styles_index: usize,

    pub fill_buffer_index: Vec<(u32, u32, i32)>,
    pub stroke_buffer_index: Vec<(u32, u32, i32)>,
}
