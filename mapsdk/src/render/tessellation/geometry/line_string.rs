use glam::Vec2;

use crate::render::tessellation::{StrokeVertexIndex, Tessellations};

pub fn tessellate_line_string(line_string: &geo::LineString) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    {
        let vertices: Vec<[f32; 2]> = line_string
            .coords()
            .map(|v| [v.x as f32, v.y as f32])
            .collect();

        let mut stroke_vertices: Vec<[f32; 5]> = Vec::new();
        let mut stroke_indices: Vec<u16> = Vec::new();

        for i in 0..vertices.len() {
            let vertex = vertices[i];

            let vec_prev: Vec2;
            let vec_next: Vec2;

            if line_string.is_closed() && (i == 0 || i == vertices.len() - 1) {
                let vertex_prev = vertices[vertices.len() - 2];
                let vertex_next = vertices[1];

                vec_prev = Vec2 {
                    x: vertex[0] - vertex_prev[0],
                    y: vertex[1] - vertex_prev[1],
                }
                .normalize_or_zero();
                vec_next = Vec2 {
                    x: vertex_next[0] - vertex[0],
                    y: vertex_next[1] - vertex[1],
                }
                .normalize_or_zero();
            } else {
                if i == 0 {
                    vec_prev = Vec2 {
                        x: vertices[1][0] - vertex[0],
                        y: vertices[1][1] - vertex[1],
                    }
                    .normalize_or_zero();
                    vec_next = vec_prev.clone();
                } else if i == vertices.len() - 1 {
                    vec_prev = Vec2 {
                        x: vertex[0] - vertices[i - 1][0],
                        y: vertex[1] - vertices[i - 1][1],
                    }
                    .normalize_or_zero();
                    vec_next = vec_prev.clone();
                } else {
                    let vertex_prev = vertices[i - 1];
                    let vertex_next = vertices[i + 1];

                    vec_prev = Vec2 {
                        x: vertex[0] - vertex_prev[0],
                        y: vertex[1] - vertex_prev[1],
                    }
                    .normalize_or_zero();
                    vec_next = Vec2 {
                        x: vertex_next[0] - vertex[0],
                        y: vertex_next[1] - vertex[1],
                    }
                    .normalize_or_zero();
                };
            }

            let vec_d = (vec_next + vec_prev).normalize_or_zero();
            let vec_n = [-vec_d.y, vec_d.x];
            let angle = vec_next.angle_to(vec_prev);

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

        output.strokes.push(StrokeVertexIndex {
            vertices: stroke_vertices,
            indices: stroke_indices,
        });
    }

    output
}

pub fn tessellate_multi_line_string(multi_line_string: &geo::MultiLineString) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for line_string in multi_line_string.iter() {
        let line_string_tessellations = tessellate_line_string(&line_string);
        output.strokes.extend(line_string_tessellations.strokes);
    }

    output
}
