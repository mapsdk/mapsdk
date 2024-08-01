use std::f32::consts::PI;

use lyon::{
    math::Point,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};

pub fn tessellate_circle(
    center: &geo::Coord,
    radius: f32,
    z: f32,
    flatten: usize,
) -> VertexBuffers<[f32; 3], u16> {
    let mut output: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();

    {
        let mut tessellator = FillTessellator::new();

        let mut buffers_builder = BuffersBuilder::new(&mut output, |vertex: FillVertex| {
            [vertex.position().x, vertex.position().y, z]
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
    }

    output
}
