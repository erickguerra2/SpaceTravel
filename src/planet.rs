use glam::Vec3;
use crate::utils::v3;

pub struct Planet {
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
    pub scale: f32,
    pub color: raylib::prelude::Color,
    pub shader: crate::renderer::PlanetShaderKind,
    pub angle_orbit: f32,
    pub angle_rot: f32,
    // trail of previous positions for drawing orbital path
    pub trail: Vec<Vec3>,
    pub trail_max: usize,
}

impl Planet {
    pub fn new(
        _name: &str,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
        scale: f32,
        color: raylib::prelude::Color,
    ) -> Self {
        Self {
            orbit_radius,
            orbit_speed,
            rotation_speed,
            scale,
            color,
            shader: crate::renderer::PlanetShaderKind::Default,
            angle_orbit: 0.0,
            angle_rot: 0.0,
            trail: Vec::new(),
            trail_max: 128,
        }
    }

    pub fn with_shader(
        _name: &str,
        orbit_radius: f32,
        orbit_speed: f32,
        rotation_speed: f32,
        scale: f32,
        color: raylib::prelude::Color,
        shader: crate::renderer::PlanetShaderKind,
    ) -> Self {
        Self {
            orbit_radius,
            orbit_speed,
            rotation_speed,
            scale,
            color,
            shader,
            angle_orbit: 0.0,
            angle_rot: 0.0,
            trail: Vec::new(),
            trail_max: 128,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.angle_orbit += self.orbit_speed * dt;
        self.angle_rot += self.rotation_speed * dt;
        // record position in trail (keep recent positions up to trail_max)
        let pos = self.position();
        self.trail.push(pos);
        if self.trail.len() > self.trail_max {
            let remove = self.trail.len() - self.trail_max;
            self.trail.drain(0..remove);
        }
    }

    pub fn position(&self) -> Vec3 {
        v3(
            self.orbit_radius * self.angle_orbit.cos(),
            0.0,
            self.orbit_radius * self.angle_orbit.sin(),
        )
    }
}
