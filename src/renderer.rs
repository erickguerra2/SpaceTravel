use raylib::prelude::*;
use glam::{Vec2, Vec3, Vec4, Mat4, Quat};

use crate::camera::Camera;
use crate::skybox::Skybox;
use crate::object::ObjMesh;
use crate::utils::clamp01;

/// Tipos de shader para planetas
#[derive(Copy, Clone)]
pub enum PlanetShaderKind {
    Sun,
    Earth,
    SuperEarth,
    Volcanic,
    Ice,
    Gas,
    Default,
}

pub struct SoftwareRenderer {
    pub width: i32,
    pub height: i32,
    /// RGBA u8 buffer (width * height * 4)
    pub color: Vec<u8>,
    /// Z-buffer (depth); valores pequeños = cerca
    depth: Vec<f32>,
}

impl SoftwareRenderer {
    pub fn new(width: i32, height: i32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            color: vec![0; size * 4],
            depth: vec![1.0; size],
        }
    }

    pub fn clear(&mut self, c: Color) {
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            let idx = i * 4;
            self.color[idx] = c.r;
            self.color[idx + 1] = c.g;
            self.color[idx + 2] = c.b;
            self.color[idx + 3] = c.a;
            self.depth[i] = 1.0;
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, z: f32, rgba: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;

        if z < self.depth[idx] {
            self.depth[idx] = z;
            let base = idx * 4;
            self.color[base] = rgba[0];
            self.color[base + 1] = rgba[1];
            self.color[base + 2] = rgba[2];
            self.color[base + 3] = rgba[3];
        }
    }

    fn put_pixel_bg(&mut self, x: i32, y: i32, rgba: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        let base = idx * 4;
        self.color[base] = rgba[0];
        self.color[base + 1] = rgba[1];
        self.color[base + 2] = rgba[2];
        self.color[base + 3] = rgba[3];
    }

    pub fn blit_to(&self, d: &mut RaylibDrawHandle) {
        // Escala el framebuffer al tamaño de la ventana haciendo muestreo por vecino.
        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();

        for sy in 0..screen_h {
            for sx in 0..screen_w {
                let src_x = (sx * self.width / screen_w) as i32;
                let src_y = (sy * self.height / screen_h) as i32;
                let idx = (src_y * self.width + src_x) as usize;
                let base = idx * 4;
                let col = Color {
                    r: self.color[base],
                    g: self.color[base + 1],
                    b: self.color[base + 2],
                    a: self.color[base + 3],
                };
                d.draw_pixel(sx, sy, col);
            }
        }
    }

    pub fn draw_skybox(&mut self, cam: &Camera, sky: &Skybox) {
        let w = self.width;
        let h = self.height;

        let fov_y = cam.fov_y.to_radians();
        let aspect = cam.aspect;
        let tan_fov_y = (fov_y * 0.5).tan();

        let forward = cam.forward();
        let up_cam = cam.up();
        let right = forward.cross(up_cam).normalize();
        let up_ortho = right.cross(forward).normalize();

        for y in 0..h {
            for x in 0..w {
                let ndc_x = ((x as f32 + 0.5) / w as f32) * 2.0 - 1.0;
                let ndc_y = 1.0 - ((y as f32 + 0.5) / h as f32) * 2.0;

                let cam_y = ndc_y * tan_fov_y;
                let cam_x = ndc_x * tan_fov_y * aspect;

                let dir = (right * cam_x + up_ortho * cam_y + forward).normalize();

                let col = sky.sample(dir);
                let r = (clamp01(col.x) * 255.0) as u8;
                let g = (clamp01(col.y) * 255.0) as u8;
                let b = (clamp01(col.z) * 255.0) as u8;

                self.put_pixel_bg(x, y, [r, g, b, 255]);
            }
        }
    }

    fn project_vertex(&self, v: Vec3, mvp: &Mat4) -> Option<(Vec2, f32)> {
        let p = *mvp * Vec4::new(v.x, v.y, v.z, 1.0);
        if p.w.abs() < 1e-6 {
            return None;
        }

        let ndc = p / p.w;

        if ndc.z < -1.0 || ndc.z > 1.0 {
            return None;
        }

        let x = (ndc.x * 0.5 + 0.5) * self.width as f32;
        let y = (-ndc.y * 0.5 + 0.5) * self.height as f32;

        let depth = ndc.z * 0.5 + 0.5;

        Some((Vec2::new(x, y), depth))
    }

    fn project_point(&self, world: Vec3, cam: &Camera) -> Option<Vec2> {
        let view = cam.view_matrix();
        let proj = cam.proj_matrix();
        let mvp = proj * view * Mat4::IDENTITY;
        self.project_vertex(world, &mvp).map(|(p, _)| p)
    }

    /// Public helper to get screen coordinates for a world point using the camera.
    pub fn world_to_screen(&self, world: Vec3, cam: &Camera) -> Option<Vec2> {
        self.project_point(world, cam)
    }

    fn draw_line_2d(&mut self, p0: Vec2, p1: Vec2, rgba: [u8; 4]) {
        let mut x0 = p0.x as i32;
        let mut y0 = p0.y as i32;
        let x1 = p1.x as i32;
        let y1 = p1.y as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_pixel(x0, y0, 1.0, rgba);

            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Órbita en el plano XZ
    pub fn draw_orbit(&mut self, radius: f32, cam: &Camera, color: Color) {
        if radius <= 0.0 {
            return;
        }
        let segments = 256;
        let rgba = [color.r, color.g, color.b, 200];

        // Draw the orbit thicker by drawing a few concentric rings (small radial offsets)
        for ring in [-0.03_f32, 0.0, 0.03_f32].iter() {
            let mut last: Option<Vec2> = None;
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let angle = t * std::f32::consts::TAU;

                let r = radius + ring;
                let world = Vec3::new(r * angle.cos(), 0.0, r * angle.sin());

                if let Some(p) = self.project_point(world, cam) {
                    if let Some(prev) = last {
                        self.draw_line_2d(prev, p, rgba);
                    }
                    last = Some(p);
                } else {
                    last = None;
                }
            }
        }
    }

    /// Mesh sombreado con shader de planeta
    pub fn draw_mesh_shaded(
        &mut self,
        mesh: &ObjMesh,
        pos: Vec3,
        scale: f32,
        base_color: Color,
        cam: &Camera,
        shader: PlanetShaderKind,
        light_dir: Vec3,
    ) {
        let model = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::IDENTITY,
            pos,
        );

        let view = cam.view_matrix();
        let proj = cam.proj_matrix();
        let mvp = proj * view * model;

        let light_dir = light_dir.normalize();

        for tri in &mesh.indices {
            // Defensive: skip triangles with indices out of range to avoid panics
            if tri[0] >= mesh.vertices.len() || tri[1] >= mesh.vertices.len() || tri[2] >= mesh.vertices.len() {
                continue;
            }
            let v0 = mesh.vertices[tri[0]];
            let v1 = mesh.vertices[tri[1]];
            let v2 = mesh.vertices[tri[2]];

            let w0 = model.transform_point3(v0);
            let w1 = model.transform_point3(v1);
            let w2 = model.transform_point3(v2);

            let n = (w1 - w0).cross(w2 - w0).normalize();

            if let (Some(p0), Some(p1), Some(p2)) = (
                self.project_vertex(v0, &mvp),
                self.project_vertex(v1, &mvp),
                self.project_vertex(v2, &mvp),
            ) {
                self.raster_triangle_shaded(
                    p0, p1, p2,
                    w0, w1, w2,
                    n,
                    base_color,
                    shader,
                    light_dir,
                );
            }
        }
    }


    /// Versión que acepta una rotación alrededor del eje Y (yaw) en radianes.
    pub fn draw_mesh_shaded_rot(
        &mut self,
        mesh: &ObjMesh,
        pos: Vec3,
        scale: f32,
        yaw: f32,
        base_color: Color,
        cam: &Camera,
        shader: PlanetShaderKind,
        light_dir: Vec3,
    ) {
        let model = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_rotation_y(yaw),
            pos,
        );

        let view = cam.view_matrix();
        let proj = cam.proj_matrix();
        let mvp = proj * view * model;

        let light_dir = light_dir.normalize();

        for tri in &mesh.indices {
            // Defensive: skip triangles with indices out of range to avoid panics
            if tri[0] >= mesh.vertices.len() || tri[1] >= mesh.vertices.len() || tri[2] >= mesh.vertices.len() {
                continue;
            }
            let v0 = mesh.vertices[tri[0]];
            let v1 = mesh.vertices[tri[1]];
            let v2 = mesh.vertices[tri[2]];

            let w0 = model.transform_point3(v0);
            let w1 = model.transform_point3(v1);
            let w2 = model.transform_point3(v2);

            let n = (w1 - w0).cross(w2 - w0).normalize();

            if let (Some(p0), Some(p1), Some(p2)) = (
                self.project_vertex(v0, &mvp),
                self.project_vertex(v1, &mvp),
                self.project_vertex(v2, &mvp),
            ) {
                self.raster_triangle_shaded(
                    p0, p1, p2,
                    w0, w1, w2,
                    n,
                    base_color,
                    shader,
                    light_dir,
                );
            }
        }
    }
    fn raster_triangle_shaded(
        &mut self,
        p0: (Vec2, f32),
        p1: (Vec2, f32),
        p2: (Vec2, f32),
        w0: Vec3,
        w1: Vec3,
        w2: Vec3,
        normal: Vec3,
        base_color: Color,
        shader: PlanetShaderKind,
        light_dir: Vec3,
    ) {
        let (v0, z0) = p0;
        let (v1, z1) = p1;
        let (v2, z2) = p2;

        let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
        let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((self.width - 1) as f32) as i32;
        let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
        let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((self.height - 1) as f32) as i32;

        let area = edge_function(v0, v1, v2);
        if area.abs() < 1e-6 {
            return;
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

                let w_a = edge_function(v1, v2, p);
                let w_b = edge_function(v2, v0, p);
                let w_c = edge_function(v0, v1, p);

                if (w_a >= 0.0 && w_b >= 0.0 && w_c >= 0.0) ||
                   (w_a <= 0.0 && w_b <= 0.0 && w_c <= 0.0) {

                    let w0n = w_a / area;
                    let w1n = w_b / area;
                    let w2n = w_c / area;

                    let z = w0n * z0 + w1n * z1 + w2n * z2;

                    let world_pos = w0 * w0n + w1 * w1n + w2 * w2n;

                    let lambert = 0.0_f32.max(normal.dot(-light_dir));

                    let rgba = shade_planet(shader, base_color, world_pos, lambert);
                    self.put_pixel(x, y, z, rgba);
                }
            }
        }
    }

    /// Additive blend into color buffer (clamps at 255). Ignores depth.
    fn add_blend_pixel(&mut self, x: i32, y: i32, add: [u8;4]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height { return; }
        let idx = (y * self.width + x) as usize;
        let base = idx * 4;
        let r = (self.color[base] as u16 + add[0] as u16).min(255) as u8;
        let g = (self.color[base + 1] as u16 + add[1] as u16).min(255) as u8;
        let b = (self.color[base + 2] as u16 + add[2] as u16).min(255) as u8;
        let a = (self.color[base + 3] as u16 + add[3] as u16).min(255) as u8;
        self.color[base] = r;
        self.color[base + 1] = g;
        self.color[base + 2] = b;
        self.color[base + 3] = a;
    }

    /// Draw an additive glow around a projected sun center. Uses the projected
    /// radius computed from `scale` to approximate screen-space halo size.
    pub fn draw_sun_glow(&mut self, center: Vec3, scale: f32, cam: &Camera) {
        // Project center
        if let Some(screen) = self.project_point(center, cam) {
            // Try to compute a screen-space radius by projecting a point offset by scale
            let probe = center + Vec3::new(scale, 0.0, 0.0);
            let screen_radius = if let Some(p2) = self.project_point(probe, cam) {
                ((p2 - screen).length()).max(8.0)
            } else {
                (scale * 10.0).min((self.width as f32) * 0.5)
            };

            let int_rad = screen_radius.ceil() as i32;
            let glow_col = [255u8, 200u8, 120u8, 120u8];

            let cx = screen.x.round() as i32;
            let cy = screen.y.round() as i32;

            for oy in -int_rad..=int_rad {
                let y = cy + oy;
                if y < 0 || y >= self.height { continue; }
                for ox in -int_rad..=int_rad {
                    let x = cx + ox;
                    if x < 0 || x >= self.width { continue; }
                    let dist = ((ox * ox + oy * oy) as f32).sqrt();
                    if dist > screen_radius { continue; }
                    // falloff: 1.0 at center -> 0.0 at radius
                    let fall = 1.0 - (dist / screen_radius);
                    let fall = fall.powf(1.5);
                    let add = [
                        (glow_col[0] as f32 * fall) as u8,
                        (glow_col[1] as f32 * fall) as u8,
                        (glow_col[2] as f32 * fall) as u8,
                        (glow_col[3] as f32 * fall) as u8,
                    ];
                    self.add_blend_pixel(x, y, add);
                }
            }
        }
    }
}

fn shade_planet(
    shader: PlanetShaderKind,
    base: Color,
    world_pos: Vec3,
    lambert: f32,
) -> [u8; 4] {
    let mut r = base.r as f32 / 255.0;
    let mut g = base.g as f32 / 255.0;
    let mut b = base.b as f32 / 255.0;

    match shader {
        PlanetShaderKind::Sun => {
            let d = world_pos.length();
            let t = (d / 3.5).min(1.0);
            r = lerp(1.0, 1.0, 1.0 - t);
            g = lerp(0.9, 0.6, t);
            b = lerp(0.3, 0.0, t);
            let glow = 0.5 + 0.5 * lambert;
            r *= glow;
            g *= glow;
            b *= glow;
        }
        PlanetShaderKind::Earth => {
            let lat = world_pos.y;
            let noise = (world_pos.x * 0.7).sin() * (world_pos.z * 0.5).cos();
            let mix_val = lat * 0.4 + noise * 0.6;

            let ocean = Vec3::new(0.0, 0.2, 0.7);
            let land  = Vec3::new(0.0, 0.5, 0.1);
            let ice   = Vec3::new(0.8, 0.8, 0.9);

            let base_col = if lat.abs() > 1.0 {
                ice
            } else if mix_val > 0.0 {
                land
            } else {
                ocean
            };

            r = base_col.x;
            g = base_col.y;
            b = base_col.z;

            // Simulate thin atmosphere: slightly brighten at rim (fresnel-like)
            let view_dir = world_pos.normalize();
            let rim = 0.3 * (1.0 - view_dir.dot(world_pos.normalize()).abs());
            let diffuse = 0.15 + 0.75 * lambert + rim;
            r *= diffuse;
            g *= diffuse;
            b *= diffuse;
        }
        PlanetShaderKind::SuperEarth => {
            // SuperEarth: yellowish/tan Earth-like with oceans and continents
            let lat = world_pos.y;
            let noise = (world_pos.x * 0.7).sin() * (world_pos.z * 0.5).cos();
            let mix_val = lat * 0.4 + noise * 0.6;

            let ocean = Vec3::new(0.1, 0.3, 0.6);
            let land  = Vec3::new(0.7, 0.65, 0.3);
            let ice   = Vec3::new(0.85, 0.82, 0.8);

            let base_col = if lat.abs() > 1.0 {
                ice
            } else if mix_val > 0.0 {
                land
            } else {
                ocean
            };

            r = base_col.x;
            g = base_col.y;
            b = base_col.z;

            let view_dir = world_pos.normalize();
            let rim = 0.4 * (1.0 - view_dir.dot(world_pos.normalize()).abs());
            let diffuse = 0.15 + 0.75 * lambert + rim;
            r *= diffuse;
            g *= diffuse;
            b *= diffuse;
        }
        PlanetShaderKind::Volcanic => {
            // Volcanic: dark red/maroon surface
            let noise = (world_pos.x * 2.5).sin() * (world_pos.z * 3.0).cos();
            let dark_red = Vec3::new(0.35, 0.08, 0.05);
            let lighter_red = Vec3::new(0.55, 0.12, 0.08);
            let t = (noise * 0.5 + 0.5).clamp(0.0, 1.0);
            let col = dark_red * (1.0 - t) + lighter_red * t;
            let diffuse = 0.3 + 0.65 * lambert;
            r = col.x * diffuse;
            g = col.y * diffuse;
            b = col.z * diffuse;
        }
        PlanetShaderKind::Ice => {
            let noise = (world_pos.z * 2.0).sin() * (world_pos.y * 3.0).cos();
            let ice1 = Vec3::new(0.7, 0.9, 1.0);
            let ice2 = Vec3::new(0.4, 0.7, 0.9);
            let t = (noise * 0.5 + 0.5).clamp(0.0, 1.0);
            let col = ice1 * t + ice2 * (1.0 - t);
            // Ice is brighter with bluish tint and some specular
            let spec = 0.2 * lambert.powf(8.0);
            let diffuse = 0.5 + 0.5 * lambert;
            r = col.x * diffuse + spec;
            g = col.y * diffuse + spec * 0.9;
            b = col.z * diffuse + spec * 0.6;
        }
        PlanetShaderKind::Gas => {
            // Gas giants: add visible horizontal bands + subtle turbulence
            let band = (world_pos.y * 6.0).sin();
            let turb = ((world_pos.x * 8.0).sin() * (world_pos.z * 6.0).cos()) * 0.2;
            let col1 = Vec3::new(0.95, 0.85, 0.7);
            let col2 = Vec3::new(0.7, 0.55, 0.4);
            let t = (band * 0.5 + 0.5 + turb).clamp(0.0, 1.0);
            let col = col1 * t + col2 * (1.0 - t);
            let diffuse = 0.25 + 0.75 * lambert;
            // Slight atmospheric haze darkening
            let haze = 1.0 - 0.15 * (world_pos.length() / 2.0).min(1.0);
            r = col.x * diffuse * haze;
            g = col.y * diffuse * haze;
            b = col.z * diffuse * haze;
        }
        PlanetShaderKind::Default => {
            let diffuse = 0.3 + 0.7 * lambert;
            r *= diffuse;
            g *= diffuse;
            b *= diffuse;
        }
    }

    [
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn edge_function(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

impl SoftwareRenderer {
    /// Dibuja un anillo plano alrededor de `center` en el plano XZ entre `inner_r` y `outer_r`.
    /// El anillo se proyecta y escribe valores de profundidad para que respete el z-buffer.
    pub fn draw_ring(&mut self, center: Vec3, inner_r: f32, outer_r: f32, cam: &crate::camera::Camera, color: Color) {
        let segments = 128; // Reduced from 256 to avoid excessive computation
        for i in 0..segments {
            let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p0_inner = Vec3::new(center.x + inner_r * a0.cos(), center.y, center.z + inner_r * a0.sin());
            let p0_outer = Vec3::new(center.x + outer_r * a0.cos(), center.y, center.z + outer_r * a0.sin());
            let p1_inner = Vec3::new(center.x + inner_r * a1.cos(), center.y, center.z + inner_r * a1.sin());
            let p1_outer = Vec3::new(center.x + outer_r * a1.cos(), center.y, center.z + outer_r * a1.sin());

            // Build view/proj matrices once to avoid recomputation
            let view = cam.view_matrix();
            let proj = cam.proj_matrix();
            let vp = proj * view;

            if let (Some((s0i, z0i)), Some((s0o, z0o)), Some((s1i, z1i)), Some((s1o, z1o))) = (
                self.project_vertex(p0_inner, &vp),
                self.project_vertex(p0_outer, &vp),
                self.project_vertex(p1_inner, &vp),
                self.project_vertex(p1_outer, &vp)
            ) {
                // Validate screen coordinates are within reasonable bounds
                let is_valid = |p: Vec2| -> bool {
                    p.x.is_finite() && p.y.is_finite() && 
                    p.x >= -1000.0 && p.x <= (self.width + 1000) as f32 &&
                    p.y >= -1000.0 && p.y <= (self.height + 1000) as f32
                };

                if is_valid(s0i) && is_valid(s0o) && is_valid(s1i) && is_valid(s1o) &&
                   z0i.is_finite() && z0o.is_finite() && z1i.is_finite() && z1o.is_finite() {
                    let rgba = [color.r, color.g, color.b, color.a];
                    // Draw the ring segments
                    self.draw_line_between_with_depth(s0i, z0i, s0o, z0o, rgba);
                    self.draw_line_between_with_depth(s1i, z1i, s1o, z1o, rgba);
                    self.draw_line_between_with_depth(s0o, z0o, s1o, z1o, rgba);
                    self.draw_line_between_with_depth(s0i, z0i, s1i, z1i, rgba);
                }
            }
        }
    }

    fn draw_line_between_with_depth(&mut self, a: Vec2, za: f32, b: Vec2, zb: f32, rgba: [u8;4]) {
        let dx = (b.x - a.x).abs();
        let dy = (b.y - a.y).abs();
        let steps = (dx.max(dy)).max(1.0).min(500.0) as i32; // Cap steps to avoid huge loops
        for i in 0..=steps {
            let t = i as f32 / (steps as f32).max(1.0);
            let x = (a.x + (b.x - a.x) * t).round() as i32;
            let y = (a.y + (b.y - a.y) * t).round() as i32;
            let z = za + (zb - za) * t;
            self.put_pixel(x, y, z, rgba);
        }
    }
}
