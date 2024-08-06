use std::{collections::HashMap, error::Error};

use geo::{BoundingRect, Coord, CoordsIter, Geometry, Rect};
use nanoid::nanoid;

use crate::JsonValue;

pub mod style;

#[derive(Clone, Debug)]
pub struct Feature {
    id: String,
    shape: Shape,
    attrs: Option<HashMap<String, JsonValue>>,
    bbox: Rect,
}

impl Feature {
    pub fn new(id: &str, shape: Shape, attrs: Option<HashMap<String, JsonValue>>) -> Self {
        let bbox = shape.bbox();

        Self {
            id: id.to_string(),
            shape,
            attrs,
            bbox,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn attrs(&self) -> &Option<HashMap<String, JsonValue>> {
        &self.attrs
    }

    pub fn bbox(&self) -> &Rect {
        &self.bbox
    }
}

pub enum Features {
    Single(Feature),
    Collection(Vec<Feature>),
}

impl From<Feature> for Features {
    fn from(feature: Feature) -> Self {
        Features::Single(feature)
    }
}

impl From<Vec<Feature>> for Features {
    fn from(features: Vec<Feature>) -> Self {
        Features::Collection(features)
    }
}

impl Features {
    pub fn from_geojson(geojson_str: &str) -> Result<Features, Box<dyn Error>> {
        match geojson_str.parse::<geojson::GeoJson>()? {
            geojson::GeoJson::Geometry(geojson_geom) => {
                Ok(
                    Feature::new(&nanoid!(), Shape::Geometry(geojson_geom.try_into()?), None)
                        .into(),
                )
            }
            geojson::GeoJson::Feature(geojson_feature) => {
                let feature = feature_from_geojson_feature(geojson_feature)?;
                Ok(feature.into())
            }
            geojson::GeoJson::FeatureCollection(geojson_feature_collection) => {
                let mut features: Vec<Feature> = Vec::new();
                for geojson_feature in geojson_feature_collection.features.into_iter() {
                    features.push(feature_from_geojson_feature(geojson_feature)?);
                }
                Ok(features.into())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Geometry(Geometry),
    Circle { center: Coord, radius: f64 },
}

impl Shape {
    pub fn bbox(&self) -> Rect {
        match self {
            Shape::Geometry(geom) => {
                if let Some(rect) = geom.bounding_rect() {
                    rect
                } else {
                    let mut xmin = f64::MAX;
                    let mut ymin = f64::MAX;
                    let mut xmax = f64::MIN;
                    let mut ymax = f64::MIN;

                    geom.coords_iter().for_each(|coord| {
                        xmin = xmin.min(coord.x);
                        ymin = ymin.min(coord.y);
                        xmax = xmax.max(coord.x);
                        ymax = ymax.max(coord.y);
                    });

                    Rect::new(Coord { x: xmin, y: ymin }, Coord { x: xmax, y: ymax })
                }
            }
            &Shape::Circle { center, radius } => Rect::new(
                Coord {
                    x: center.x - radius,
                    y: center.y - radius,
                },
                Coord {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            ),
        }
    }

    pub fn is_points(&self) -> bool {
        match self {
            Shape::Geometry(geom) => match geom {
                Geometry::Point(_) | Geometry::MultiPoint(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

fn feature_from_geojson_feature(
    geojson_feature: geojson::Feature,
) -> Result<Feature, Box<dyn Error>> {
    let id = if let Some(geojson_id) = geojson_feature.id {
        match geojson_id {
            geojson::feature::Id::String(id) => id,
            geojson::feature::Id::Number(id) => id.to_string(),
        }
    } else {
        nanoid!()
    };

    let geom = geojson_feature
        .geometry
        .ok_or("No geometry in feature")?
        .try_into()?;

    let attrs = if let Some(properties) = geojson_feature.properties {
        Some(HashMap::from_iter(properties.into_iter()))
    } else {
        None
    };

    Ok(Feature::new(&id, Shape::Geometry(geom), attrs))
}

#[cfg(test)]
mod tests {
    use geo::Point;

    use super::*;

    #[test]
    fn test_features_from_geojson() {
        match Features::from_geojson(
            r#"
            {
              "type": "Feature",
              "id": 1,
              "properties": {"name": "zero", "value": 0},
              "geometry": {
                "type": "Point",
                "coordinates": [0.0, 0.0]
              }
            }
            "#,
        )
        .unwrap()
        {
            Features::Single(feature) => {
                assert_eq!(feature.id(), "1");

                match feature.shape {
                    Shape::Geometry(geom) => {
                        assert_eq!(
                            TryInto::<Point>::try_into(geom).unwrap(),
                            Point::new(0.0, 0.0)
                        );
                    }
                    _ => assert!(false),
                }

                assert_eq!(feature.attrs.as_ref().unwrap()["name"], "zero");
                assert_eq!(feature.attrs.as_ref().unwrap()["value"], 0);
            }
            _ => assert!(false),
        }

        match Features::from_geojson(
            r#"
            {
              "type": "FeatureCollection",
              "features": [
              {
                "type": "Feature",
                "properties": {"name": "zero", "value": 0},
                "geometry": {
                  "type": "Point",
                  "coordinates": [0.0, 0.0]
                }
              }
              ]
            }
            "#,
        )
        .unwrap()
        {
            Features::Collection(features) => {
                assert_eq!(features.len(), 1);
            }
            _ => assert!(false),
        }
    }
}
