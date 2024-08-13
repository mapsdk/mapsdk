use std::{collections::BTreeMap, error::Error};

use geo::*;
use nanoid::nanoid;

use crate::feature::{Feature, Shape};

#[derive(Debug, Clone)]
pub struct VectorTile {
    pub layers: BTreeMap<String, VectorTileLayer>,
}

impl VectorTile {
    pub fn from_data(data: Vec<u8>, tile_bbox: &Rect) -> Result<Self, Box<dyn Error>> {
        let reader = mvt_reader::Reader::new(data)?;

        let tile_x = tile_bbox.min().x as f32;
        let tile_y = tile_bbox.max().y as f32;
        let tile_x_factor = (tile_bbox.max().x - tile_bbox.min().x) as f32 / 4096.0;
        let tile_y_factor = (tile_bbox.max().y - tile_bbox.min().y) as f32 / 4096.0;

        let mut layers: BTreeMap<String, VectorTileLayer> = BTreeMap::new();

        let layer_names = reader.get_layer_names()?;
        for (i, name) in layer_names.iter().enumerate() {
            let layer = VectorTileLayer {
                features: reader
                    .get_features(i)?
                    .into_iter()
                    .map(|f| {
                        let id = nanoid!();

                        let geom = translate_vector_tile_geometry(
                            &f.geometry,
                            tile_x,
                            tile_y,
                            tile_x_factor,
                            tile_y_factor,
                        );
                        let shape = Shape::Geometry(geom);

                        let attrs = if let Some(properties) = f.properties {
                            Some(
                                properties
                                    .iter()
                                    .map(|(k, v)| (k.clone(), serde_json::Value::from(v.clone())))
                                    .collect(),
                            )
                        } else {
                            None
                        };

                        Feature::<f32>::new(&id, shape, attrs)
                    })
                    .collect(),
            };

            layers.insert(name.to_string(), layer);
        }

        Ok(Self { layers })
    }
}

#[derive(Debug, Clone)]
pub struct VectorTileLayer {
    pub features: Vec<Feature<f32>>,
}

fn translate_vector_tile_geometry(
    tile_geom: &Geometry<f32>,
    tile_x: f32,
    tile_y: f32,
    tile_x_factor: f32,
    tile_y_factor: f32,
) -> Geometry<f32> {
    let coord_fn = |coord: Coord<f32>| -> Coord<f32> {
        let x = tile_x + coord.x * tile_x_factor;
        let y = tile_y - coord.y * tile_y_factor;
        Coord { x, y }
    };

    translate_geometry(tile_geom, &coord_fn)
}

fn translate_geometry(
    geometry: &Geometry<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> Geometry<f32> {
    match geometry {
        Geometry::Point(point) => translate_point(point, &coord_fn).into(),
        Geometry::MultiPoint(multi_point) => translate_multi_point(multi_point, &coord_fn).into(),
        Geometry::Line(line) => translate_line(line, &coord_fn).into(),
        Geometry::LineString(line_string) => translate_line_string(line_string, &coord_fn).into(),
        Geometry::MultiLineString(multi_line_string) => {
            translate_multi_line_string(multi_line_string, &coord_fn).into()
        }
        Geometry::Polygon(polygon) => translate_polygon(polygon, &coord_fn).into(),
        Geometry::MultiPolygon(multi_polygon) => {
            translate_multi_polygon(multi_polygon, &coord_fn).into()
        }
        Geometry::Rect(rect) => translate_rect(rect, &coord_fn).into(),
        Geometry::Triangle(triangle) => translate_triangle(triangle, &coord_fn).into(),
        Geometry::GeometryCollection(geometry_collection) => Geometry::GeometryCollection(
            translate_geometry_collection(geometry_collection, &coord_fn),
        ),
    }
}

fn translate_point(point: &Point<f32>, coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>) -> Point<f32> {
    let coord = coord_fn(point.0);
    coord.into()
}

fn translate_multi_point(
    multi_point: &MultiPoint<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> MultiPoint<f32> {
    let points: Vec<Point<f32>> = multi_point
        .iter()
        .map(|p| translate_point(p, coord_fn))
        .collect();
    MultiPoint::new(points)
}

fn translate_line(line: &Line<f32>, coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>) -> Line<f32> {
    let coord_start = coord_fn(line.start);
    let coord_end = coord_fn(line.end);
    Line::new(coord_start, coord_end)
}

fn translate_line_string(
    line_string: &LineString<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> LineString<f32> {
    let coords: Vec<Coord<f32>> = line_string.coords_iter().map(coord_fn).collect();
    LineString::new(coords)
}

fn translate_multi_line_string(
    multi_line_string: &MultiLineString<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> MultiLineString<f32> {
    let line_strings: Vec<LineString<f32>> = multi_line_string
        .iter()
        .map(|l| translate_line_string(l, coord_fn))
        .collect();
    MultiLineString::new(line_strings)
}

fn translate_polygon(
    polygon: &Polygon<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> Polygon<f32> {
    let exterior = translate_line_string(&polygon.exterior(), coord_fn);
    let interiors = polygon
        .interiors()
        .iter()
        .map(|l| translate_line_string(l, coord_fn))
        .collect();
    Polygon::new(exterior, interiors)
}

fn translate_multi_polygon(
    multi_polygon: &MultiPolygon<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> MultiPolygon<f32> {
    let polygons: Vec<Polygon<f32>> = multi_polygon
        .iter()
        .map(|l| translate_polygon(l, coord_fn))
        .collect();
    MultiPolygon::new(polygons)
}

fn translate_rect(rect: &Rect<f32>, coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>) -> Rect<f32> {
    let min = coord_fn(rect.min());
    let max = coord_fn(rect.max());
    Rect::new(min, max)
}

fn translate_triangle(
    triangle: &Triangle<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> Triangle<f32> {
    let v0 = coord_fn(triangle.0);
    let v1 = coord_fn(triangle.1);
    let v2 = coord_fn(triangle.2);
    Triangle::new(v0, v1, v2)
}

fn translate_geometry_collection(
    geometry_collection: &GeometryCollection<f32>,
    coord_fn: &dyn Fn(Coord<f32>) -> Coord<f32>,
) -> GeometryCollection<f32> {
    let collections: Vec<Geometry<f32>> = geometry_collection
        .iter()
        .map(|g| translate_geometry(g, coord_fn))
        .collect();
    GeometryCollection::new_from(collections)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;

    #[test]
    fn test_vector_tile() {
        let mut data_reader = File::open("../assets/mvt.pbf").unwrap();
        let mut data = Vec::new();
        data_reader.read_to_end(&mut data).unwrap();

        let vt = VectorTile::from_data(
            data,
            &Rect::new(
                Coord {
                    x: -20037508.34278924,
                    y: -20037508.34278924,
                },
                Coord {
                    x: 20037508.34278924,
                    y: 20037508.34278924,
                },
            ),
        )
        .unwrap();
        assert_eq!(vt.layers.len(), 3);
        assert_eq!(
            vt.layers.keys().cloned().collect::<Vec<_>>(),
            ["centroids", "countries", "geolines"]
        );

        println!("---{:?}", vt.layers.get("centroids").unwrap().features);
    }
}
