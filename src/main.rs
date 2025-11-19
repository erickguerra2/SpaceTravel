mod renderer;
mod camera;
mod planet;
mod object;
mod skybox;
mod warp;
mod movement;
mod texture;
mod utils;
mod math;

use raylib::prelude::*;
use renderer::{SoftwareRenderer, PlanetShaderKind};
use camera::Camera;
use skybox::Skybox;
use planet::Planet;
use movement::update_third_person;
use warp::Warp;
use utils::*;
use glam::Vec3;

fn keep_camera_outside_planets(cam: &mut Camera, ship_pos: &mut glam::Vec3, planets: &[Planet]) {
    for p in planets {
        let center = p.position();
        let to_ship = *ship_pos - center;
        let dist = to_ship.length();
        let safe = p.scale * 2.5 + 4.0;

        if dist < safe {
            let dir = to_ship.normalize();
            // Move the ship out to the safe distance so it doesn't penetrate the planet.
            *ship_pos = center + dir * safe;
            // Make camera target the ship; camera update will smoothly follow.
            cam.target = *ship_pos;
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("SpaceTravel - Proyecto Final")
        .build();

    rl.set_target_fps(60);

    // Tamaño del framebuffer
    let w = 1280;
    let h = 720;

    let mut renderer = SoftwareRenderer::new(w, h);

    // Cámara inicial
    let mut cam = Camera::new(
        v3(0.0, 8.0, 35.0),
        // inicialmente miramos hacia delante; luego ajustamos target a la nave
        v3(0.0, 0.0, 30.0),
    );

    // Skybox
    let sky = Skybox::load();

    // Modelos
    let sphere = object::ObjMesh::load("assets/models/sphere.obj");
    let ship = object::ObjMesh::load("assets/models/ship.obj");

    // Warp system
    let mut warp = Warp::new();

    // Planetas con escala / color / shader
    let mut planets: Vec<Planet> = vec![
        Planet::with_shader("Sol", 0.0, 0.0, 0.0, 4.0, Color::YELLOW, PlanetShaderKind::Sun),
        // Increased orbit radii to give more spacing between planets
        Planet::with_shader("Tierra", 18.0, 0.7, 1.5, 1.3, Color::BLUE, PlanetShaderKind::Earth),
        Planet::with_shader("Volcanico", 28.0, 0.5, 1.1, 1.5, Color::RED, PlanetShaderKind::Volcanic),
        Planet::with_shader("Helado", 40.0, 0.42, 0.9, 1.6, Color::SKYBLUE, PlanetShaderKind::Ice),
        Planet::with_shader("Gaseoso", 55.0, 0.35, 0.7, 2.7, Color::BEIGE, PlanetShaderKind::Gas),
        // SuperTierra: una super tierra con anillos rojizos (usa shader Earth con color amarilloso)
        Planet::with_shader("SuperTierra", 72.0, 0.28, 0.9, 2.0, Color::new(200, 180, 80, 255), PlanetShaderKind::SuperEarth),
    ];

    let earth_idx = 1;

    let light_dir = Vec3::new(1.0, -0.4, -0.2);

    // Initial ship position: in front of camera
    // Slightly lower the ship vertical offset so the camera is clearly above it
    let mut ship_pos = cam.pos + cam.forward() * 5.0 + v3(0.0, -0.2, 0.0);
    // Make camera orbit state consistent with this starting configuration
    cam.target = ship_pos;
    cam.set_orbit_from_pos();
    // Set a comfortable starting distance / pitch so camera is not directly above
    cam.distance = 10.0;
    cam.pitch = 0.35; // camera above target looking down
    cam.update_pos_from_orbit();

    let mut prev_warp_active = false;
    let mut frame_count: u64 = 0;
    while !rl.window_should_close() {
        frame_count += 1;
        if frame_count % 60 == 0 {
            eprintln!("frame {}", frame_count);
        }
        let dt = rl.get_frame_time();

        // Actualizar controles (mueve la nave y actualiza la cámara en 3ª persona)
        update_third_person(&rl, &mut cam, &mut ship_pos, dt, warp.is_active());

        // Warp 1–5
        for (i, key) in [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR,
            KeyboardKey::KEY_FIVE,
        ]
        .iter()
        .enumerate()
        {
            if rl.is_key_pressed(*key) && i < planets.len() {
                // Prevent starting a new warp while one is active (spamming keys
                // rapidly can cause instability). If a warp is already active,
                // ignore the request.
                if warp.is_active() {
                    eprintln!("Warp request ignored because warp is already active");
                    continue;
                }
                let center = planets[i].position();
                let mut dir = center - cam.pos;
                if dir.length_squared() < 1e-6 {
                    // fallback direction if we're exactly at center: point along -Z
                    dir = Vec3::new(0.0, 0.0, -1.0);
                } else {
                    dir = dir.normalize();
                }
                let safe = center - dir * (planets[i].scale * 2.5 + 6.0);
                // Iniciar warp desde la posición actual hacia `safe`, mirando al centro del planeta
                eprintln!("Starting warp to planet {}: safe={:?}", i, safe);
                warp.start(cam.pos, safe, cam.target, center);
            }
        }

        // Apply warp (if active) which controls camera position/target.
        warp.apply(dt, &mut cam);

        // If warp just finished this frame, move the ship to the warp target so
        // the camera and ship stay in sync and we don't snap back to the previous
        // ship position.
        let warp_active = warp.is_active();
        if prev_warp_active && !warp_active {
            // warp ended this frame
            // place ship at warp end safe position (not at planet center)
            ship_pos = warp.end_position();
            // update camera target to ship; keep orbit values (yaw/pitch/distance) stable
            // so camera doesn't snap or get stuck. The third-person update will smoothly
            // position the camera around the new target using the current orbit params.
            cam.target = ship_pos;
        }

        // Keep ship outside planets, but do not modify the ship/camera while warp is active
        if !warp_active {
            keep_camera_outside_planets(&mut cam, &mut ship_pos, &planets);
        }

        prev_warp_active = warp_active;

        // Actualizar órbitas
        for p in planets.iter_mut() {
            p.update(dt);
        }

        // LIMPIAR Y RENDERIZAR
        renderer.time = rl.get_time() as f32;
        renderer.clear(Color::BLACK);

        // Skybox
        renderer.draw_skybox(&cam, &sky);

        // Órbitas (más visibles)
        for p in &planets {
            renderer.draw_orbit(p.orbit_radius, &cam, Color::LIGHTGRAY);
        }

        // Dibujar planetas con shaders bonitos
        for p in &planets {
            let pos = p.position();
            renderer.draw_mesh_shaded(
                &sphere,
                pos,
                p.scale,
                p.color,
                &cam,
                p.shader,
                light_dir,
            );
            // Dibujar anillos solo para planetas específicos
            match p.shader {
                crate::renderer::PlanetShaderKind::Gas => {
                    renderer.draw_ring(pos, p.scale * 1.6, p.scale * 3.0, &cam, Color::new(200, 180, 140, 200));
                }
                crate::renderer::PlanetShaderKind::SuperEarth => {
                    // SuperEarth has reddish/lava-like rings
                    renderer.draw_ring(pos, p.scale * 1.25, p.scale * 2.5, &cam, Color::new(180, 60, 30, 200));
                }
                _ => {}
            }
        }
        eprintln!("frame {}: after planets", frame_count);

        // (Temporal) Desactivado: halo solar para depuración del crash.
        // if let Some(sun) = planets.get(0) {
        //     renderer.draw_sun_glow(sun.position(), sun.scale, &cam);
        // }

        // Luna que orbita la Tierra
        let earth_pos = planets[earth_idx].position();
        let moon_angle = rl.get_time() as f32;
        let moon_pos = earth_pos + v3(3.0 * moon_angle.cos(), 0.0, 3.0 * moon_angle.sin());

        renderer.draw_mesh_shaded(
            &sphere,
            moon_pos,
            0.5,
            Color::LIGHTGRAY,
            &cam,
            PlanetShaderKind::Ice,
            light_dir,
        );
        eprintln!("frame {}: after moon", frame_count);

        // Dibujar la nave en `ship_pos`. Rotamos 180deg para corregir orientación del modelo.
        renderer.draw_mesh_shaded_rot(
            &ship,
            ship_pos,
            0.9,
            std::f32::consts::PI,
            Color::WHITE,
            &cam,
            PlanetShaderKind::Default,
            light_dir,
        );
        eprintln!("frame {}: after ship", frame_count);

        // Presentar framebuffer a pantalla completa
        {
            eprintln!("frame {}: before begin_drawing", frame_count);
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            renderer.blit_to(&mut d);
            eprintln!("frame {}: after blit_to", frame_count);

            // Draw orbital trails (project recent world positions to screen and draw native lines)
            for p in &planets {
                if p.trail.len() < 2 { continue; }
                let mut last_screen: Option<(i32,i32)> = None;
                for wp in &p.trail {
                    if let Some(sp) = renderer.world_to_screen(*wp, &cam) {
                        let px = sp.x as i32;
                        let py = sp.y as i32;
                        if let Some((lx, ly)) = last_screen {
                            d.draw_line(lx, ly, px, py, Color::new(p.color.r, p.color.g, p.color.b, 160));
                        }
                        last_screen = Some((px, py));
                    } else {
                        last_screen = None;
                    }
                }
            }

            // Sun fill + glow (additive) - keep after planet rasterization so it blends
            if let Some(sun) = planets.get(0) {
                // draw the halo/glow (keep only glow for now to avoid recent crash)
                renderer.draw_sun_glow(sun.position(), sun.scale, &cam);
            }

            // Draw ship (rotated 180deg)
            renderer.draw_mesh_shaded_rot(
                &ship,
                ship_pos,
                0.9,
                std::f32::consts::PI,
                Color::WHITE,
                &cam,
                PlanetShaderKind::Default,
                light_dir,
            );

            d.draw_text("WASD para mover | SPACE/CTRL subir/bajar | 1–5 Warp", 10, 10, 20, Color::WHITE);
        }
    }
}
