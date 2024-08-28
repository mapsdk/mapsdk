use std::{collections::HashMap, time::Instant};

use geo::{polygon, Coord, Polygon, Rect};
use glam::{DQuat, DVec3};

use crate::{
    layer::Layer,
    map::MapOptions,
    render::{InterRenderers, MapRenderer},
};

pub struct MapContext {
    pub map_options: MapOptions,
    pub map_state: MapState,

    pub layers: HashMap<String, Box<dyn Layer>>,

    pub animating: bool,

    pub map_renderer: MapRenderer,
    pub inter_renderers: InterRenderers,
}

impl MapContext {
    pub fn new(
        map_options: &MapOptions,
        map_renderer: MapRenderer,
        inter_renderers: InterRenderers,
    ) -> Self {
        let map_state = MapState {
            center: map_options.center.clone(),
            pitch: map_options.pitch,
            yaw: map_options.yaw,
            zoom: map_options.zoom,
            zoom_res: map_options.tiling.get_resolution(map_options.zoom),
            ..Default::default()
        };

        Self {
            map_options: map_options.clone(),
            map_state,

            layers: HashMap::new(),

            animating: false,

            map_renderer,
            inter_renderers,
        }
    }

    pub fn redraw(&mut self) {
        let instant = Instant::now();

        log::debug!(
            "Redraw map, center: {:?}, zoom resolution: {:?}, map resolution ratio: {:?}, pitch: {:?}, yaw: {:?}",
            self.map_state.center,
            self.map_state.zoom_res,
            self.map_state.map_res_ratio,
            self.map_state.pitch,
            self.map_state.yaw,
        );

        if self.map_state.view_bounds_seq != self.map_state.view_seq {
            if let Some(view_bounds) = self.calc_view_bounds() {
                self.map_state.view_bounds = view_bounds;
            }
            self.map_state.view_bounds_seq = self.map_state.view_seq;
        }

        if !self.animating {
            for (id, layer) in &mut self.layers {
                log::debug!("Update layer [{}]", id);
                layer.update(
                    &self.map_options,
                    &self.map_state,
                    &mut self.map_renderer,
                    &mut self.inter_renderers,
                );
            }
        }

        self.map_renderer.render(
            &self.map_options,
            &self.map_state,
            &mut self.inter_renderers,
        );

        log::info!(
            "MapContext::redraw(update+render) elapsed: {:?}",
            instant.elapsed()
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.map_state.map_res_ratio =
            self.map_options.tiling.tile_size() as f64 / width.min(height) as f64;

        self.map_renderer.resize(
            width,
            height,
            &self.map_options,
            &self.map_state,
            &mut self.inter_renderers,
        );

        self.map_state.view_seq += 1;
    }

    pub fn set_center(&mut self, center: Coord) {
        self.map_state.center = center;
        self.map_state.view_seq += 1;
    }

    pub fn set_pitch_yaw(&mut self, pitch: f64, yaw: f64) {
        self.map_state.pitch = pitch;
        self.map_state.yaw = yaw;

        self.map_renderer.set_pitch_yaw(
            pitch,
            yaw,
            &self.map_options,
            &self.map_state,
            &mut self.inter_renderers,
        );

        self.map_state.view_seq += 1;
    }

    pub fn set_zoom_res(&mut self, zoom_res: f64, update_zoom: bool) {
        let zoom_res_max = self
            .map_options
            .tiling
            .get_resolution(self.map_options.zoom_min);
        let zoom_res_min = self
            .map_options
            .tiling
            .get_resolution(self.map_options.zoom_max);

        let new_zoom_res = if self.animating {
            zoom_res
        } else {
            zoom_res.clamp(zoom_res_min, zoom_res_max)
        };

        self.map_state.zoom_res = new_zoom_res;
        if update_zoom {
            self.map_state.zoom = self.map_options.tiling.get_closest_lower_zoom(new_zoom_res);
        }

        self.map_state.view_seq += 1;
    }

    pub fn to_map(&self, screen_coord: &Coord) -> Option<Coord> {
        let screen_center_x = self.map_renderer.width() as f64 / 2.0;
        let screen_center_y = self.map_renderer.height() as f64 / 2.0;

        let v0 = DVec3::new(
            screen_coord.x - screen_center_x,
            screen_center_y - screen_coord.y,
            0.0,
        );
        let v1 = DQuat::from_axis_angle(DVec3::X, self.map_state.pitch.to_radians()) * v0;
        let v2 = DQuat::from_axis_angle(DVec3::Z, self.map_state.yaw.to_radians()) * v1;

        let target = self.map_renderer.camera().target().as_dvec3();
        let eye = self.map_renderer.camera().eye().as_dvec3();

        let v = target + v2;

        // Ray-Plane Intersection
        // Reference: https://www.cs.princeton.edu/courses/archive/fall00/cs426/lectures/raycast/sld017.htm
        let ray = v - eye;
        let t = (target.dot(DVec3::Z) - v.dot(DVec3::Z)) / ray.dot(DVec3::Z);
        let p = v + ray * t;

        let map_center = self.map_state.center;
        let map_res = self.map_state.zoom_res * self.map_state.map_res_ratio;

        let map_coord = map_center
            + Coord {
                x: p.x.into(),
                y: p.y.into(),
            } * map_res;

        Some(map_coord)
    }

    pub fn to_screen(&self, map_coord: &Coord) -> Option<Coord> {
        let r0 = DQuat::from_axis_angle(DVec3::X, self.map_state.pitch.to_radians());
        let s0 = r0 * DVec3::Z;
        let r1 = DQuat::from_axis_angle(DVec3::Z, self.map_state.yaw.to_radians());
        let s1 = r1 * s0;

        let map_center = self.map_state.center;
        let map_res = self.map_state.zoom_res * self.map_state.map_res_ratio;

        let mp = DVec3::new(
            (map_coord.x - map_center.x) / map_res,
            (map_coord.y - map_center.y) / map_res,
            0.0,
        );

        let target = self.map_renderer.camera().target().as_dvec3();
        let eye = self.map_renderer.camera().eye().as_dvec3();

        // Ray-Plane Intersection
        // Reference: https://www.cs.princeton.edu/courses/archive/fall00/cs426/lectures/raycast/sld017.htm
        let ray = target - eye;
        let t = (target.dot(s1) - mp.dot(s1)) / ray.dot(s1);
        let p = mp + ray * t;

        let v0 = p - target;
        let v1 = DQuat::from_axis_angle(DVec3::Z, -self.map_state.yaw.to_radians()) * v0;
        let v2 = DQuat::from_axis_angle(DVec3::X, -self.map_state.pitch.to_radians()) * v1;

        let screen_center_x = self.map_renderer.width() as f64 / 2.0;
        let screen_center_y = self.map_renderer.height() as f64 / 2.0;

        let screen_coord = Coord {
            x: screen_center_x + v2.x as f64,
            y: screen_center_y - v2.y as f64,
        };

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

        let zoom_res = self.map_state.zoom_res;
        let new_zoom_res = (zoom_res / scalar).clamp(zoom_res_min, zoom_res_max);

        self.map_state.zoom_res = new_zoom_res;
        if new_zoom_res != zoom_res {
            self.map_state.zoom = self.map_options.tiling.get_closest_lower_zoom(new_zoom_res);

            let center = self.map_state.center;
            self.map_state.center = *coord + (center - *coord) * (new_zoom_res / zoom_res);
        } else if new_zoom_res == zoom_res_max && scalar < 1.0 {
            let center = self.map_state.center;
            let origin_center = self.map_options.center;
            self.map_state.center = center + (origin_center - center) * (1.0 - scalar).powf(0.5);
        }

        self.map_state.view_seq += 1;
    }

    fn calc_view_bounds(&self) -> Option<Polygon> {
        let center = self.map_state.center;
        let map_res = self.map_state.zoom_res * self.map_state.map_res_ratio;

        let hw = self.map_renderer.width() as f64 / 2.0;
        let hh = self.map_renderer.height() as f64 / 2.0;

        let r = DQuat::from_axis_angle(DVec3::Z, self.map_state.yaw.to_radians());

        let hd = (hw * hw + hh * hh).sqrt();

        let alpha = (hh / hd).atan();
        let theta = alpha - self.map_state.pitch.to_radians();

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

    pub layers_order: Vec<String>,

    view_bounds: Polygon,
    view_bounds_seq: u64,
    view_seq: u64,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            center: Coord { x: 0.0, y: 0.0 },
            map_res_ratio: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            zoom: 0,
            zoom_res: 1.0,

            layers_order: Vec::new(),

            view_bounds: Rect::new(Coord { x: -1.0, y: -1.0 }, Coord { x: 1.0, y: 1.0 })
                .to_polygon(),
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
