use lyon::{
    math::point,
    path::Path,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};

use crate::render::tessellation::{
    line_string::tessellate_line_string, FillVertexIndex, Tessellations,
};

pub fn tessellate_polygon(polygon: &geo::Polygon, tolerance: f32) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    {
        let mut fill_tessellation: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();

        let mut buffers_builder =
            BuffersBuilder::new(&mut fill_tessellation, |vertex: FillVertex| {
                [vertex.position().x, vertex.position().y]
            });

        let mut path_builder = Path::builder();

        {
            for (i, coord) in polygon.exterior().coords().enumerate() {
                let p = point(coord.x as f32, coord.y as f32);

                if i == 0 {
                    path_builder.begin(p);
                } else {
                    path_builder.line_to(p);
                }
            }
            path_builder.close();

            for hole in polygon.interiors() {
                for (i, coord) in hole.coords().enumerate() {
                    let p = point(coord.x as f32, coord.y as f32);

                    if i == 0 {
                        path_builder.begin(p);
                    } else {
                        path_builder.line_to(p);
                    }
                }
                path_builder.close();
            }
        }

        let path = path_builder.build();

        let options = FillOptions::tolerance(tolerance);

        let mut tessellator = FillTessellator::new();
        if let Err(err) = tessellator.tessellate_path(&path, &options, &mut buffers_builder) {
            log::error!("{:?}", err);
        }

        output.fills.push(FillVertexIndex {
            vertices: fill_tessellation.vertices,
            indices: fill_tessellation.indices,
        });
    }

    {
        let exterior = polygon.exterior();
        let exterior_tessellations = tessellate_line_string(&exterior, true);
        output.strokes.extend(exterior_tessellations.strokes);

        for interior in polygon.interiors() {
            let interior_tessellations = tessellate_line_string(&interior, true);
            output.strokes.extend(interior_tessellations.strokes);
        }
    }

    output
}
