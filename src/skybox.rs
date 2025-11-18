use glam::Vec3;
use crate::texture::TextureCPU;

pub struct Skybox {
    pub nx: TextureCPU,
    pub px: TextureCPU,
    pub ny: TextureCPU,
    pub py: TextureCPU,
    pub nz: TextureCPU,
    pub pz: TextureCPU,
}

impl Skybox {
    pub fn load() -> Self {
        Self {
            nx: TextureCPU::from_file("assets/skybox/left.png"),
            px: TextureCPU::from_file("assets/skybox/right.png"),
            ny: TextureCPU::from_file("assets/skybox/bottom.png"),
            py: TextureCPU::from_file("assets/skybox/top.png"),
            nz: TextureCPU::from_file("assets/skybox/back.png"),
            pz: TextureCPU::from_file("assets/skybox/front.png"),
        }
    }

    pub fn sample(&self, dir: Vec3) -> Vec3 {
        let abs = dir.abs();

        let (tex, u, v) = if abs.x >= abs.y && abs.x >= abs.z {
            if dir.x > 0.0 { (&self.px, -dir.z/abs.x,  dir.y/abs.x) }
            else           { (&self.nx,  dir.z/abs.x,  dir.y/abs.x) }
        } else if abs.y >= abs.z {
            if dir.y > 0.0 { (&self.py, dir.x/abs.y, -dir.z/abs.y) }
            else           { (&self.ny, dir.x/abs.y,  dir.z/abs.y) }
        } else {
            if dir.z > 0.0 { (&self.pz, dir.x/abs.z,  dir.y/abs.z) }
            else           { (&self.nz, -dir.x/abs.z, dir.y/abs.z) }
        };

        let u = 0.5 * (u + 1.0);
        let v = 0.5 * (v + 1.0);

        tex.sample(u, v)
    }
}
