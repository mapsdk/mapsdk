use crate::render::tessellation::{FillVertexIndex, Tessellations};

pub fn tessellate_point(point: &geo::Point) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    {
        let vertex: [f32; 2] = [point.x() as f32, point.y() as f32];

        let fill_vertices: Vec<[f32; 2]> =
            vec![vertex, vertex.clone(), vertex.clone(), vertex.clone()];
        let fill_indices = vec![0, 2, 1, 1, 2, 3];

        output.fills.push(FillVertexIndex {
            vertices: fill_vertices,
            indices: fill_indices,
        });
    }

    output
}

pub fn tessellate_multi_point(multi_point: &geo::MultiPoint) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for point in multi_point.iter() {
        let point_tessellations = tessellate_point(&point);
        output.fills.extend(point_tessellations.fills);
    }

    output
}
