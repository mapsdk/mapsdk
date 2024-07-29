use geo::{polygon, BoundingRect, Intersects};

use crate::{
    geo::Coord,
    map::context::MapState,
    tiling::{TileId, Tiling},
};

pub fn tile_ids_in_view(map_state: &MapState, tiling: &Tiling) -> Vec<TileId> {
    let mut tile_ids = Vec::new();

    let z = map_state.zoom;

    if let Some(view_rect) = map_state.view_bounds().bounding_rect() {
        if let (Some(lt), Some(lb), Some(rt), Some(rb)) = (
            tiling.get_tile_id(z, &Coord::new(view_rect.min().x, view_rect.max().y)),
            tiling.get_tile_id(z, &Coord::new(view_rect.min().x, view_rect.min().y)),
            tiling.get_tile_id(z, &Coord::new(view_rect.max().x, view_rect.max().y)),
            tiling.get_tile_id(z, &Coord::new(view_rect.max().x, view_rect.min().y)),
        ) {
            let max_x_y = tiling.get_max_x_y(z);

            let min_x = lt.x.min(lb.x).min(rt.x).min(rb.x).max(0);
            let max_x = lt.x.max(lb.x).max(rt.x).max(rb.x).min(max_x_y);
            let min_y = lt.y.min(lb.y).min(rt.y).min(rb.y).max(0);
            let max_y = lt.y.max(lb.y).max(rt.y).max(rb.y).min(max_x_y);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let tile_id = TileId { z, x, y };
                    if let Some(bbox) = tiling.get_tile_bbox(&tile_id) {
                        let tile_polygon = polygon![
                        (x: bbox.xmin, y: bbox.ymax),
                        (x: bbox.xmin, y: bbox.ymin),
                        (x: bbox.xmax, y: bbox.ymin),
                        (x: bbox.xmax, y: bbox.ymax),
                        ];

                        if tile_polygon.intersects(map_state.view_bounds()) {
                            tile_ids.push(tile_id);
                        }
                    }
                }
            }
        }
    }

    tile_ids
}

pub fn format_tile_url(
    url_template: &str,
    z: usize,
    x: i32,
    y: i32,
    subdomains: &Option<Vec<impl ToString>>,
) -> String {
    let mut url = String::from(url_template)
        .replace("{z}", &z.to_string())
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string());

    if let Some(subdomains) = subdomains {
        let count = subdomains.len();
        if count > 0 {
            let i = (x + y).abs() as usize % count;
            url = url.replace("{s}", &subdomains[i].to_string());
        }
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tile_url() {
        assert_eq!(
            format_tile_url(
                "http://{s}.tile.osm.org/{z}/{x}/{y}.png",
                2,
                0,
                1,
                &None::<Vec<String>>
            ),
            "http://{s}.tile.osm.org/2/0/1.png"
        );
        assert_eq!(
            format_tile_url(
                "http://{s}.tile.osm.org/{z}/{x}/{y}.png",
                2,
                0,
                1,
                &None::<Vec<String>>
            ),
            "http://{s}.tile.osm.org/2/0/1.png"
        );
        assert_eq!(
            format_tile_url(
                "http://{s}.tile.osm.org/{z}/{x}/{y}.png",
                2,
                0,
                1,
                &vec!["a", "b", "c"].into()
            ),
            "http://b.tile.osm.org/2/0/1.png"
        );
    }
}
