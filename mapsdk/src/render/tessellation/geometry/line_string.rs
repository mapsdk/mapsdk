use crate::{
    render::tessellation::{StrokeVertexIndex, Tessellations},
    CoordType,
};

pub fn tessellate_line_string<T: CoordType>(line_string: &geo::LineString<T>) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    let vertices: Vec<[f32; 2]> = line_string
        .coords()
        .map(|v| [CoordType::to_f32(v.x), CoordType::to_f32(v.y)])
        .collect();

    let is_closed = line_string.is_closed();

    {
        let mut stroke_vertices: Vec<[f32; 7]> = Vec::new();
        let mut stroke_indices: Vec<u16> = Vec::new();
        let mut stroke_index_offset: u16 = 0;

        let seg_count = vertices.len() - 1;
        for i in 0..seg_count {
            // Line start join
            {
                if i > 0 || (i == 0 && is_closed) {
                    let v = vertices[i];
                    let v_prev = if i == 0 {
                        vertices[seg_count - 1]
                    } else {
                        vertices[i - 1]
                    };
                    let v_next = vertices[i + 1];

                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 0.3]);

                    stroke_indices.extend_from_slice(&[
                        stroke_index_offset,
                        stroke_index_offset + 2,
                        stroke_index_offset + 1,
                    ]);

                    stroke_index_offset += 1;
                }
            }

            // Straight line segment
            {
                // Line start
                {
                    let v = vertices[i];
                    let v_prev = if i == 0 {
                        if is_closed {
                            vertices[seg_count - 1]
                        } else {
                            vertices[0]
                        }
                    } else {
                        vertices[i - 1]
                    };
                    let v_next = vertices[i + 1];

                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 0.1]);
                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 0.2]);
                }

                // Line end
                {
                    let v = vertices[i + 1];
                    let v_prev = vertices[i];
                    let v_next = if i == seg_count - 1 {
                        if is_closed {
                            vertices[1]
                        } else {
                            vertices[i + 1]
                        }
                    } else {
                        vertices[i + 2]
                    };

                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 1.1]);
                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 1.2]);
                }

                stroke_indices.extend_from_slice(&[
                    stroke_index_offset,
                    stroke_index_offset + 1,
                    stroke_index_offset + 2,
                    stroke_index_offset + 2,
                    stroke_index_offset + 1,
                    stroke_index_offset + 3,
                ]);

                stroke_index_offset += 4;
            }

            // Line end join
            {
                if i < seg_count - 1 || (i == seg_count - 1 && is_closed) {
                    let v = vertices[i + 1];
                    let v_prev = vertices[i];
                    let v_next = if i == seg_count - 1 {
                        vertices[1]
                    } else {
                        vertices[i + 2]
                    };

                    stroke_vertices
                        .push([v[0], v[1], v_prev[0], v_prev[1], v_next[0], v_next[1], 1.3]);

                    stroke_indices.extend_from_slice(&[
                        stroke_index_offset - 2,
                        stroke_index_offset - 1,
                        stroke_index_offset,
                    ]);

                    if i == seg_count - 1 {
                        stroke_indices.extend_from_slice(&[
                            stroke_index_offset - 2,
                            stroke_index_offset,
                            0,
                        ]);
                    } else {
                        stroke_indices.extend_from_slice(&[
                            stroke_index_offset - 2,
                            stroke_index_offset,
                            stroke_index_offset + 1,
                        ]);
                    }

                    stroke_index_offset += 1;
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

pub fn tessellate_multi_line_string<T: CoordType>(
    multi_line_string: &geo::MultiLineString<T>,
) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for line_string in multi_line_string.iter() {
        let line_string_tessellations = tessellate_line_string(&line_string);
        output.strokes.extend(line_string_tessellations.strokes);
    }

    output
}
