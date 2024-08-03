use std::{cmp::Ordering, f32::consts::PI};

use glam::Vec2;
use lyon::{
    math::Point,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};

use crate::render::tessellation::Tessellation;

pub fn tessellate_circle(center: &geo::Coord, radius: f32, flatten: usize) -> Tessellation {
    let mut output: Tessellation = Tessellation::new();

    {
        let mut tessellator = FillTessellator::new();

        let mut fill_tessellation: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
        let mut buffers_builder =
            BuffersBuilder::new(&mut fill_tessellation, |vertex: FillVertex| {
                [vertex.position().x, vertex.position().y]
            });

        let tolerance = radius - radius * (PI / 2.0_f32.powf(flatten as f32)).cos();
        let options = FillOptions::tolerance(tolerance);
        if let Err(err) = tessellator.tessellate_circle(
            Point::new(center.x as f32, center.y as f32),
            radius,
            &options,
            &mut buffers_builder,
        ) {
            log::error!("{:?}", err);
        }

        output.fill_vertices = fill_tessellation.vertices;
        output.fill_indices = fill_tessellation.indices;
    }

    {
        let mut outline_vertices_angles: Vec<_> = output
            .fill_vertices
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

        let vec_center = Vec2 {
            x: center.x as f32,
            y: center.y as f32,
        };

        let mut stroke_vertices: Vec<[f32; 5]> = Vec::new();
        let mut stroke_indices: Vec<u16> = Vec::new();
        for i in 0..outline_vertices.len() {
            let vertex = outline_vertices[i];
            let vec_n = (Vec2 {
                x: vertex[0],
                y: vertex[1],
            } - vec_center)
                .normalize_or_zero();

            let angle = if i == 0 || i == outline_vertices.len() - 1 {
                0.0
            } else {
                let vertex_prev = outline_vertices[i - 1];
                let vertex_next = outline_vertices[i + 1];

                let vec_prev = Vec2 {
                    x: vertex[0] - vertex_prev[0],
                    y: vertex[1] - vertex_prev[1],
                };
                let vec_next = Vec2 {
                    x: vertex_next[0] - vertex[0],
                    y: vertex_next[1] - vertex[1],
                };

                vec_next.angle_to(vec_prev)
            };

            stroke_vertices.push([vertex[0], vertex[1], vec_n[0], vec_n[1], angle]);
            stroke_vertices.push([vertex[0], vertex[1], -vec_n[0], -vec_n[1], -angle]);

            if i > 0 {
                let offset = 2 * i as u16 - 2;
                stroke_indices.extend_from_slice(&[
                    offset,
                    offset + 2,
                    offset + 1,
                    offset + 1,
                    offset + 2,
                    offset + 3,
                ]);
            }
        }

        output.stroke_vertices = stroke_vertices;
        output.stroke_indices = stroke_indices;
    }

    output
}
