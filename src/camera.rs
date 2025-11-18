use glam::{Vec3, Mat4};
use crate::utils::*;

pub struct Camera {
    pub pos: Vec3,
    pub target: Vec3,
    pub fov_y: f32,
    pub aspect: f32,
    // Orbit controls (spherical coordinates around target)
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Camera {
    pub fn new(pos: Vec3, target: Vec3) -> Self {
        let mut cam = Self {
            pos,
            target,
            fov_y: 60.0,
            aspect: 640.0 / 360.0,
            yaw: 0.0,
            pitch: 0.0,
            distance: 10.0,
        };
        cam.set_orbit_from_pos();
        cam
    }

    // Initialize yaw/pitch/distance from current pos/target
    pub fn set_orbit_from_pos(&mut self) {
        let offset = self.pos - self.target;
        self.distance = offset.length().max(0.001);
        self.pitch = (offset.y / self.distance).asin();
        self.yaw = offset.z.atan2(offset.x);
    }

    // Compute pos from yaw/pitch/distance and current target
    pub fn update_pos_from_orbit(&mut self) {
        let cp = self.pitch.cos();
        let x = self.distance * cp * self.yaw.cos();
        let z = self.distance * cp * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        self.pos = self.target + Vec3::new(x, y, z);
    }

    pub fn forward(&self) -> Vec3 {
        (self.target - self.pos).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        look_at(self.pos, self.target, Vec3::Y)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        perspective(self.fov_y.to_radians(), self.aspect, 0.1, 2000.0)
    }
}
