use crate::{
    map::context::MapState,
    tiling::{TileId, Tiling},
};

pub fn tile_ids_in_view(map_state: &MapState, tiling: &Tiling) -> Vec<TileId> {
    let mut tile_ids = Vec::new();

    let z = map_state.zoom;
    let view_bounds = map_state.view_bounds();
    if let (Some(lt), Some(lb), Some(rt), Some(rb)) = (
        tiling.get_tile_id(z, &view_bounds.lt),
        tiling.get_tile_id(z, &view_bounds.lb),
        tiling.get_tile_id(z, &view_bounds.rt),
        tiling.get_tile_id(z, &view_bounds.rb),
    ) {
        let max_x_y = tiling.get_max_x_y(z);

        let min_x = lt.x.min(lb.x).max(0);
        let max_x = rt.x.max(rb.x).min(max_x_y);
        let min_y = lt.y.min(lb.y).max(0);
        let max_y = rt.y.max(rb.y).min(max_x_y);

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                // TODO reduce tile ids
                tile_ids.push(TileId { z, x, y });
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
