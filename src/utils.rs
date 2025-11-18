use glam::{Vec3, Vec4, Mat4};

pub fn v3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn clamp01(x: f32) -> f32 {
    if x < 0.0 { 0.0 }
    else if x > 1.0 { 1.0 }
    else { x }
}

pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    Mat4::perspective_rh(fov_y_rad, aspect, near, far)
}
