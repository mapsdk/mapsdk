use crate::{
    render::tessellation::{FillVertexIndex, Tessellations},
    CoordType,
};

pub fn tessellate_point<T: CoordType>(point: &geo::Point<T>) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    {
        let vertex: [f32; 2] = [CoordType::to_f32(point.x()), CoordType::to_f32(point.y())];

        let fill_vertices: Vec<[f32; 2]> =
            vec![vertex, vertex.clone(), vertex.clone(), vertex.clone()];
        let fill_indices = vec![0, 1, 2, 3];

        output.fills.push(FillVertexIndex {
            vertices: fill_vertices,
            indices: fill_indices,
        });
    }

    output
}

pub fn tessellate_multi_point<T: CoordType>(multi_point: &geo::MultiPoint<T>) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for point in multi_point.iter() {
        let point_tessellations = tessellate_point(&point);
        output.fills.extend(point_tessellations.fills);
    }

    output
}
