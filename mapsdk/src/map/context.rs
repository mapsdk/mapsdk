use std::{collections::BTreeMap, time::Instant};

use geo::{polygon, Polygon};
use glam::{DQuat, DVec3};

use crate::{
    geo::{Bbox, Coord},
    layer::Layer,
    map::MapOptions,
    render::Renderer,
};

pub struct MapContext {
    pub map_options: MapOptions,
    pub state: MapState,

    pub layers: BTreeMap<String, Box<dyn Layer>>,
    pub renderer: Option<Renderer>,
}

impl MapContext {
    pub fn new(map_options: &MapOptions) -> Self {
        let state = MapState {
            center: map_options.center.clone(),
            pitch: map_options.pitch,
            yaw: map_options.yaw,
            zoom: map_options.zoom,
            zoom_res: map_options.tiling.get_resolution(map_options.zoom),
            ..Default::default()
        };

        Self {
            map_options: map_options.clone(),
            state,

            layers: BTreeMap::new(),
            renderer: None,
        }
    }

    pub fn redraw(&mut self) {
        let instant = Instant::now();

        log::debug!(
            "Redraw map, center: {:?}, zoom resolution: {:?}, map resolution ratio: {:?}, pitch: {:?}, yaw: {:?}",
            self.state.center,
            self.state.zoom_res,
            self.state.map_res_ratio,
            self.state.pitch,
            self.state.yaw,
        );

        if self.state.view_bounds_seq != self.state.view_seq {
            if let Some(view_bounds) = self.calc_view_bounds() {
                self.state.view_bounds = view_bounds;
            }
            self.state.view_bounds_seq = self.state.view_seq;
        }

        if let Some(renderer) = &mut self.renderer {
            for (id, layer) in &mut self.layers {
                log::debug!("Update layer [{}]", id);
                layer.update(&self.map_options, &self.state, renderer);
            }

            renderer.render(&self.state);
        }

        log::info!(
            "MapContext::redraw(update+render) elapsed: {:?}",
            instant.elapsed()
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.state.map_res_ratio =
            self.map_options.tiling.tile_size as f64 / width.min(height) as f64;

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height, &self.state);
        }

        self.state.view_seq += 1;
    }

    pub fn set_center(&mut self, center: Coord) {
        self.state.center = center;
        self.state.view_seq += 1;
    }

    pub fn set_pitch_yaw(&mut self, pitch: f64, yaw: f64) {
        self.state.pitch = pitch;
        self.state.yaw = yaw;

        if let Some(renderer) = &mut self.renderer {
            renderer.set_pitch_yaw(pitch, yaw, &self.state);
        }

        self.state.view_seq += 1;
    }

    pub fn to_map(&self, screen_coord: &Coord) -> Option<Coord> {
        let screen_center_x = self.renderer.as_ref()?.width() as f64 / 2.0;
        let screen_center_y = self.renderer.as_ref()?.height() as f64 / 2.0;

        let v0 = DVec3::new(
            screen_coord.x - screen_center_x,
            screen_center_y - screen_coord.y,
            0.0,
        );
        let v1 = DQuat::from_axis_angle(DVec3::X, self.state.pitch.to_radians()) * v0;
        let v2 = DQuat::from_axis_angle(DVec3::Z, self.state.yaw.to_radians()) * v1;

        let target = self.renderer.as_ref()?.camera().target().as_dvec3();
        let eye = self.renderer.as_ref()?.camera().eye().as_dvec3();

        let v = target + v2;

        // Ray-Plane Intersection
        // Reference: https://www.cs.princeton.edu/courses/archive/fall00/cs426/lectures/raycast/sld017.htm
        let ray = v - eye;
        let t = (target.dot(DVec3::Z) - v.dot(DVec3::Z)) / ray.dot(DVec3::Z);
        let p = v + ray * t;

        let map_center = self.state.center;
        let map_res = self.state.zoom_res * self.state.map_res_ratio;

        let map_coord = map_center + Coord::new(p.x.into(), p.y.into()) * map_res;

        Some(map_coord)
    }

    pub fn to_screen(&self, map_coord: &Coord) -> Option<Coord> {
        let r0 = DQuat::from_axis_angle(DVec3::X, self.state.pitch.to_radians());
        let s0 = r0 * DVec3::Z;
        let r1 = DQuat::from_axis_angle(DVec3::Z, self.state.yaw.to_radians());
        let s1 = r1 * s0;

        let map_center = self.state.center;
        let map_res = self.state.zoom_res * self.state.map_res_ratio;

        let mp = DVec3::new(
            (map_coord.x - map_center.x) / map_res,
            (map_coord.y - map_center.y) / map_res,
            0.0,
        );

        let target = self.renderer.as_ref()?.camera().target().as_dvec3();
        let eye = self.renderer.as_ref()?.camera().eye().as_dvec3();

        // Ray-Plane Intersection
        // Reference: https://www.cs.princeton.edu/courses/archive/fall00/cs426/lectures/raycast/sld017.htm
        let ray = target - eye;
        let t = (target.dot(s1) - mp.dot(s1)) / ray.dot(s1);
        let p = mp + ray * t;

        let v0 = p - target;
        let v1 = DQuat::from_axis_angle(DVec3::Z, -self.state.yaw.to_radians()) * v0;
        let v2 = DQuat::from_axis_angle(DVec3::X, -self.state.pitch.to_radians()) * v1;

        let screen_center_x = self.renderer.as_ref()?.width() as f64 / 2.0;
        let screen_center_y = self.renderer.as_ref()?.height() as f64 / 2.0;

        let screen_coord = Coord::new(screen_center_x + v2.x as f64, screen_center_y - v2.y as f64);

        Some(screen_coord)
    }

    pub fn zoom_around(&mut self, coord: &Coord, scalar: f64) {
        let zoom_res_max = self
            .map_options
            .tiling
            .get_resolution(self.map_options.zoom_min);
        let zoom_res_min = self
            .map_options
            .tiling
            .get_resolution(self.map_options.zoom_max);

        let zoom_res = self.state.zoom_res;
        let new_zoom_res = (zoom_res / scalar).clamp(zoom_res_min, zoom_res_max);

        self.state.zoom_res = new_zoom_res;
        if new_zoom_res != zoom_res {
            self.state.zoom = self.map_options.tiling.get_closest_lower_zoom(new_zoom_res);

            let center = self.state.center;
            self.state.center = *coord + (center - *coord) * (new_zoom_res / zoom_res);
        } else if new_zoom_res == zoom_res_max && scalar < 1.0 {
            let center = self.state.center;
            let origin_center = self.map_options.center;
            self.state.center = center + (origin_center - center) * (1.0 - scalar).powf(0.5);
        }

        self.state.view_seq += 1;
    }

    fn calc_view_bounds(&self) -> Option<Polygon> {
        let center = self.state.center;
        let map_res = self.state.zoom_res * self.state.map_res_ratio;

        let hw = self.renderer.as_ref()?.width() as f64 / 2.0;
        let hh = self.renderer.as_ref()?.height() as f64 / 2.0;

        let r = DQuat::from_axis_angle(DVec3::Z, self.state.yaw.to_radians());

        let hd = (hw * hw + hh * hh).sqrt();

        let alpha = (hh / hd).atan();
        let theta = alpha - self.state.pitch.to_radians();

        const MAX_FACTOR: f64 = 3.0;
        let factor = if theta > 0.0 {
            (alpha.sin() / theta.sin()).min(MAX_FACTOR)
        } else {
            MAX_FACTOR
        };

        let scale = map_res * factor;

        let v_lt = r * DVec3::new(-hw, hh, 0.0) * scale;
        let v_lb = r * DVec3::new(-hw, -hh, 0.0) * scale;
        let v_rt = r * DVec3::new(hw, hh, 0.0) * scale;
        let v_rb = r * DVec3::new(hw, -hh, 0.0) * scale;

        Some(polygon![
            (x: center.x + v_lt.x, y: center.y + v_lt.y),
            (x: center.x + v_lb.x, y: center.y + v_lb.y),
            (x: center.x + v_rb.x, y: center.y + v_rb.y),
            (x: center.x + v_rt.x, y: center.y + v_rt.y),
        ])
    }
}

#[derive(Debug)]
pub struct MapState {
    pub center: Coord,
    pub map_res_ratio: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub zoom: usize,
    pub zoom_res: f64,

    view_bounds: Polygon,
    view_bounds_seq: u64,
    view_seq: u64,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            center: Coord::new(0.0, 0.0),
            map_res_ratio: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            zoom: 0,
            zoom_res: 1.0,

            view_bounds: Bbox::new(-1.0, -1.0, 1.0, 1.0).into(),
            view_bounds_seq: 0,
            view_seq: 1,
        }
    }
}

impl MapState {
    pub fn view_bounds(&self) -> &Polygon {
        &self.view_bounds
    }
}
