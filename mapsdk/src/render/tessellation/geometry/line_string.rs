use glam::Vec2;

use crate::render::tessellation::{StrokeVertexIndex, Tessellations};

pub fn tessellate_line_string(line_string: &geo::LineString) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    {
        let mut stroke_vertices: Vec<[f32; 8]> = Vec::new();
        let mut stroke_indices: Vec<u16> = Vec::new();
        let mut index_offset: u16 = 0;

        let vertices: Vec<_> = line_string
            .coords()
            .map(|v| Vec2::new(v.x as f32, v.y as f32))
            .collect();

        for i in 0..vertices.len() - 1 {
            let seg_dir = (vertices[i + 1] - vertices[i]).normalize_or_zero();
            let seg_norm = Vec2::new(-seg_dir.y, seg_dir.x);

            let prev_seg_dir = if i == 0 {
                if line_string.is_closed() {
                    (vertices[vertices.len() - 1] - vertices[vertices.len() - 2])
                        .normalize_or_zero()
                } else {
                    seg_dir
                }
            } else {
                (vertices[i] - vertices[i - 1]).normalize_or_zero()
            };
            let seg_start_join_angle = seg_dir.angle_to(prev_seg_dir);

            let next_seg_dir = if i == vertices.len() - 2 {
                if line_string.is_closed() {
                    (vertices[1] - vertices[0]).normalize_or_zero()
                } else {
                    seg_dir
                }
            } else {
                (vertices[i + 2] - vertices[i + 1]).normalize_or_zero()
            };
            let seg_end_join_angle = next_seg_dir.angle_to(seg_dir);

            let seg_start_vertex = vertices[i];
            let seg_end_vertex = vertices[i + 1];

            // Line start join
            if i > 0 || (i == 0 && line_string.is_closed()) {
                if seg_start_join_angle != 0.0 {
                    let join_vertex = vertices[i];
                    let join_dir = (prev_seg_dir + seg_dir).normalize_or_zero();
                    let join_norm =
                        Vec2::new(-join_dir.y, join_dir.x) * seg_start_join_angle.signum();
                    let join_norm_offset = join_norm / (seg_start_join_angle / 2.0).cos().abs();
                    let edge_norm = seg_norm * seg_start_join_angle.signum();

                    stroke_vertices.push([
                        join_vertex.x,
                        join_vertex.y,
                        0.0,
                        0.0,
                        join_norm_offset.x,
                        join_norm_offset.y,
                        edge_norm.x,
                        edge_norm.y,
                    ]);

                    stroke_indices.extend_from_slice(&[
                        index_offset,
                        index_offset + 1,
                        index_offset + 2,
                    ]);

                    index_offset += 1;
                }
            }

            // Straight line segment
            {
                for j in [1.0, -1.0] {
                    let offset_scale = (seg_start_join_angle / 2.0).tan().abs();
                    stroke_vertices.push([
                        seg_start_vertex.x,
                        seg_start_vertex.y,
                        seg_dir.x * offset_scale,
                        seg_dir.y * offset_scale,
                        j * seg_norm.x,
                        j * seg_norm.y,
                        j * seg_norm.x,
                        j * seg_norm.y,
                    ]);
                }

                for j in [1.0, -1.0] {
                    let offset_scale = (seg_end_join_angle / 2.0).tan().abs();
                    stroke_vertices.push([
                        seg_end_vertex.x,
                        seg_end_vertex.y,
                        -seg_dir.x * offset_scale,
                        -seg_dir.y * offset_scale,
                        j * seg_norm.x,
                        j * seg_norm.y,
                        j * seg_norm.x,
                        j * seg_norm.y,
                    ]);
                }

                stroke_indices.extend_from_slice(&[
                    index_offset,
                    index_offset + 1,
                    index_offset + 2,
                    index_offset + 2,
                    index_offset + 1,
                    index_offset + 3,
                ]);

                index_offset += 4;
            }

            // Line end join
            if i < vertices.len() - 2 || (i == vertices.len() - 2 && line_string.is_closed()) {
                if seg_end_join_angle != 0.0 {
                    let join_vertex = vertices[i + 1];
                    let join_dir = (seg_dir + next_seg_dir).normalize_or_zero();
                    let join_norm =
                        Vec2::new(-join_dir.y, join_dir.x) * seg_end_join_angle.signum();
                    let join_norm_offset = join_norm / (seg_end_join_angle / 2.0).cos().abs();
                    let edge_norm = seg_norm * seg_end_join_angle.signum();

                    stroke_vertices.push([
                        join_vertex.x,
                        join_vertex.y,
                        0.0,
                        0.0,
                        join_norm_offset.x,
                        join_norm_offset.y,
                        edge_norm.x,
                        edge_norm.y,
                    ]);

                    stroke_indices.extend_from_slice(&[
                        index_offset - 2,
                        index_offset - 1,
                        index_offset,
                    ]);

                    index_offset += 1;
                }
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
