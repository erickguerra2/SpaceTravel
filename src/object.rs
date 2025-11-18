use glam::Vec3;
use std::fs;

pub struct ObjMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<[usize; 3]>,
}

impl ObjMesh {
    pub fn load(path: &str) -> Self {
        let text = fs::read_to_string(path)
            .expect(&format!("No se pudo leer {}", path));

        let mut vertices = vec![];
        let mut indices = vec![];

        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }

            match parts[0] {
                "v" => {
                    let x: f32 = parts[1].parse().unwrap();
                    let y: f32 = parts[2].parse().unwrap();
                    let z: f32 = parts[3].parse().unwrap();
                    vertices.push(Vec3::new(x, y, z));
                }
                "f" => {
                    let mut idx = [0usize; 3];
                    for i in 0..3 {
                        let v = parts[i + 1].split('/').next().unwrap();
                        idx[i] = v.parse::<usize>().unwrap() - 1;
                    }
                    indices.push(idx);
                }
                _ => {}
            }
        }

        // Remove invalid faces (indices that reference missing vertices).
        let before = indices.len();
        indices.retain(|tri| tri[0] < vertices.len() && tri[1] < vertices.len() && tri[2] < vertices.len());
        let removed = before - indices.len();
        if removed > 0 {
            eprintln!("ObjMesh load('{}'): removed {} invalid faces (indices out of range)", path, removed);
        }

        Self { vertices, indices }
    }
}

