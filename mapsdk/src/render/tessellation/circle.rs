use std::{cmp::Ordering, f32::consts::PI};

use geo::Coord;
use glam::Vec2;
use lyon::{
    math::Point,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};

use crate::render::tessellation::{
    line_string::tessellate_line_string, FillVertexIndex, Tessellations,
};

pub fn tessellate_circle(center: &geo::Coord, radius: f32, flatten: usize) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    let mut fill_tessellation: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
    {
        let mut buffers_builder =
            BuffersBuilder::new(&mut fill_tessellation, |vertex: FillVertex| {
                [vertex.position().x, vertex.position().y]
            });

        let tolerance = radius - radius * (PI / 2.0_f32.powf(flatten as f32)).cos();
        let options = FillOptions::tolerance(tolerance);

        let mut tessellator = FillTessellator::new();
        if let Err(err) = tessellator.tessellate_circle(
            Point::new(center.x as f32, center.y as f32),
            radius,
            &options,
            &mut buffers_builder,
        ) {
            log::error!("{:?}", err);
        }
    }

    {
        let mut outline_vertices_angles: Vec<_> = fill_tessellation
            .vertices
            .iter()
            .map(|v| {
                (
                    *v,
                    Vec2 {
                        x: v[0] - center.x as f32,
                        y: v[1] - center.y as f32,
                    }
                    .angle_to(Vec2::X),
                )
            })
            .collect();
        outline_vertices_angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        let mut outline_vertices: Vec<_> = outline_vertices_angles.iter().map(|v| v.0).collect();
        outline_vertices.push(outline_vertices[0]);

        let line_string: geo::LineString = outline_vertices
            .iter()
            .map(|v| Coord {
                x: v[0] as f64,
                y: v[1] as f64,
            })
            .collect::<Vec<_>>()
            .into();
        let line_tessellations = tessellate_line_string(&line_string, true);

        output.strokes = line_tessellations.strokes;
    }

    output.fills.push(FillVertexIndex {
        vertices: fill_tessellation.vertices,
        indices: fill_tessellation.indices,
    });

    output
}
