use std::{collections::BTreeMap, error::Error};

use geo::*;
use nanoid::nanoid;

use crate::feature::{Feature, Shape};

#[derive(Debug, Clone)]
pub struct VectorTile {
    bbox: Rect,
    layers: BTreeMap<String, VectorTileLayer>,
}

impl VectorTile {
    pub fn from_data(data: Vec<u8>, tile_bbox: Rect) -> Result<Self, Box<dyn Error>> {
        let mut layers: BTreeMap<String, VectorTileLayer> = BTreeMap::new();

        let reader = mvt_reader::Reader::new(data)?;
        let layer_names = reader.get_layer_names()?;
        for (i, name) in layer_names.iter().enumerate() {
            let layer = VectorTileLayer {
                features: reader
                    .get_features(i)?
                    .into_iter()
                    .map(|f| {
                        let id = nanoid!();

                        let shape = Shape::Geometry(f.geometry);

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

        Ok(Self {
            bbox: tile_bbox,
            layers,
        })
    }

    pub fn bbox(&self) -> Rect {
        self.bbox
    }

    pub fn layers(&self) -> &BTreeMap<String, VectorTileLayer> {
        &self.layers
    }
}

#[derive(Debug, Clone)]
pub struct VectorTileLayer {
    pub features: Vec<Feature<f32>>,
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
            Rect::new(
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
    }
}
