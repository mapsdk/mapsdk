use geo::{line_string, Geometry::*};

use crate::render::tessellation::{
    geometry::{
        line_string::{tessellate_line_string, tessellate_multi_line_string},
        point::{tessellate_multi_point, tessellate_point},
        polygon::{tessellate_multi_polygon, tessellate_polygon},
    },
    Tessellations,
};

pub mod line_string;
pub mod point;
pub mod polygon;

pub fn tessellate_geometry(geom: &geo::Geometry) -> Tessellations {
    match geom {
        Point(point) => tessellate_point(point),
        Line(line) => tessellate_line_string(&line_string![line.start, line.end]),
        LineString(line_string) => tessellate_line_string(line_string),
        Polygon(polygon) => tessellate_polygon(polygon),
        MultiPoint(multi_point) => tessellate_multi_point(multi_point),
        MultiLineString(multi_line_string) => tessellate_multi_line_string(multi_line_string),
        MultiPolygon(multi_polygon) => tessellate_multi_polygon(multi_polygon),
        GeometryCollection(geometry_collection) => {
            tessellate_geometry_collection(geometry_collection)
        }
        Rect(rect) => tessellate_polygon(&rect.to_polygon()),
        Triangle(triangle) => tessellate_polygon(&triangle.to_polygon()),
    }
}

pub fn tessellate_geometry_collection(
    geometry_collection: &geo::GeometryCollection,
) -> Tessellations {
    let mut output: Tessellations = Tessellations::new();

    for geometry in geometry_collection.iter() {
        let geometry_tessellations = tessellate_geometry(&geometry);
        output.fills.extend(geometry_tessellations.fills);
        output.strokes.extend(geometry_tessellations.strokes);
    }

    output
}
