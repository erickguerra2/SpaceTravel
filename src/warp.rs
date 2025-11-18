use glam::Vec3;

pub struct Warp {
    active: bool,
    time: f32,
    start: Vec3,
    end: Vec3,
    start_target: Vec3,
    end_target: Vec3,
    duration: f32,
}

impl Warp {
    pub fn new() -> Self {
        Self {
            active: false,
            time: 0.0,
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            start_target: Vec3::ZERO,
            end_target: Vec3::ZERO,
            duration: 1.0,
        }
    }

    /// Inicia un warp desde `from` hacia `to`. `from_target` y `to_target` son
    /// los puntos que la cámara debe mirar al inicio y al final (por ejemplo el
    /// centro del planeta). La duración se calcula a partir de la distancia.
    pub fn start(&mut self, from: Vec3, to: Vec3, from_target: Vec3, to_target: Vec3) {
        self.active = true;
        self.time = 0.0;
        self.start = from;
        self.end = to;
        self.start_target = from_target;
        self.end_target = to_target;
        let dist = (to - from).length();
        // Duración razonable basada en distancia (clamp entre 0.5 y 3.0s)
        self.duration = (dist / 25.0).clamp(0.5, 3.0);
    }

    pub fn apply(&mut self, dt: f32, cam: &mut crate::camera::Camera) {
        if !self.active {
            return;
        }

        self.time += dt;
        let t = (self.time / self.duration).min(1.0);

        // Interpolación suave (ease in/out)
        let t_smooth = t * t * (3.0 - 2.0 * t);

        cam.pos = self.start.lerp(self.end, t_smooth);
        cam.target = self.start_target.lerp(self.end_target, t_smooth);

        if t >= 1.0 {
            self.active = false;
            eprintln!("Warp finished: end={:?}, end_target={:?}", self.end, self.end_target);
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the warp end position (camera-safe point) when available.
    pub fn end_position(&self) -> Vec3 {
        self.end
    }
}
