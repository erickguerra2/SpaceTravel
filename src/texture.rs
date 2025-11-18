use raylib::prelude::*;
use glam::Vec3;

pub struct TextureCPU {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>, // RGB
}

impl TextureCPU {
    pub fn from_file(path: &str) -> Self {
        let img = Image::load_image(path).expect("No se pudo cargar textura");
        let w = img.width() as u32;
        let h = img.height() as u32;

        let mut data = Vec::with_capacity((w * h * 3) as usize);

        // Image::data() retorna RGBA en raylib 5.5.1
        let pixels: &[u8] = unsafe {
            // img.data() returns a raw pointer to bytes (c_void); cast to *const u8 and create a slice
            std::slice::from_raw_parts(img.data() as *const u8, (w * h * 4) as usize)
        };
        for y in 0..h {
            for x in 0..w {
                // each pixel is 4 bytes: R,G,B,A
                let idx = (((y * w) + x) * 4) as usize;
                data.push(pixels[idx]);
                data.push(pixels[idx + 1]);
                data.push(pixels[idx + 2]);
            }
        }

        Self { w, h, data }
    }

    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let uu = (u * (self.w - 1) as f32) as usize;
        let vv = (v * (self.h - 1) as f32) as usize;
        let idx = (vv * self.w as usize + uu) * 3;

        Vec3::new(
            self.data[idx] as f32 / 255.0,
            self.data[idx + 1] as f32 / 255.0,
            self.data[idx + 2] as f32 / 255.0,
        )
    }
}
