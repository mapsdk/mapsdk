use glam::{Mat4, Vec3};

fn calc_view_proj(
    eye: &Vec3,
    target: &Vec3,
    up: &Vec3,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let view = Mat4::look_at_rh(*eye, *target, *up);
    let proj = Mat4::perspective_rh(fovy.to_radians(), aspect, near, far);
    let view_proj = proj * view;

    view_proj.to_cols_array_2d()
}

#[derive(Debug)]
pub(crate) struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,

    view_proj: [[f32; 4]; 4],
}

impl Default for Camera {
    fn default() -> Self {
        let eye = Vec3::new(0.0, 0.0, 1.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::Y;
        let aspect = 1.0;
        let fovy = 90.0;
        let near = 0.0;
        let far = f32::MAX;

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            near,
            far,

            view_proj: calc_view_proj(&eye, &target, &up, aspect, fovy, near, far),
        }
    }
}

#[allow(dead_code)]
impl Camera {
    pub fn set_eye(&mut self, x: f32, y: f32, z: f32) {
        self.eye = Vec3::new(x, y, z);
        self.update_view_proj();
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
        self.target = Vec3::new(x, y, z);
        self.update_view_proj();
    }

    pub fn set_up(&mut self, x: f32, y: f32, z: f32) {
        self.up = Vec3::new(x, y, z);
        self.update_view_proj();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_view_proj();
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.update_view_proj();
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
        self.update_view_proj();
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
        self.update_view_proj();
    }

    pub fn view_proj(&self) -> [[f32; 4]; 4] {
        self.view_proj
    }

    fn update_view_proj(&mut self) {
        self.view_proj = calc_view_proj(
            &self.eye,
            &self.target,
            &self.up,
            self.aspect,
            self.fovy,
            self.near,
            self.far,
        );
    }
}
