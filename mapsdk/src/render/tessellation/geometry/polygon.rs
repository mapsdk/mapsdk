use geo::*;
use lyon::{
    math::point,
    path::Path,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};

use crate::{
    render::tessellation::{
        geometry::line_string::tessellate_line_string, FillVertexIndex, Tessellations,
    },
    CoordType,
};

pub fn tessellate_polygon<T: CoordType>(polygon: &geo::Polygon<T>) -> Tessellations {
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
                let p = point(CoordType::to_f32(coord.x), CoordType::to_f32(coord.y));

                if i == 0 {
                    path_builder.begin(p);
                } else {
                    path_builder.line_to(p);
                }
            }
            path_builder.close();

            for hole in polygon.interiors() {
                for (i, coord) in hole.coords().enumerate() {
                    let p = point(CoordType::to_f32(coord.x), CoordType::to_f32(coord.y));

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

        let options = FillOptions::tolerance(f32::EPSILON);

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
        let exterior_tessellations = tessellate_line_string(&exterior);
        output.strokes.extend(exterior_tessellations.strokes);

        for interior in polygon.interiors() {
            let reversed_interior =
                LineString::from(interior.coords_iter().rev().collect::<Vec<_>>());
            let interior_tessellations = tessellate_line_string(&reversed_interior);
            output.strokes.extend(interior_tessellations.strokes);
        }
    }

    output
}

pub fn tessellate_multi_polygon<T: CoordType>(
    multi_polygon: &geo::MultiPolygon<T>,
) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for polygon in multi_polygon.iter() {
        let polygon_tessellations = tessellate_polygon(&polygon);
        output.fills.extend(polygon_tessellations.fills);
        output.strokes.extend(polygon_tessellations.strokes);
    }

    output
}
