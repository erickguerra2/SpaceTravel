use crate::camera::Camera;
use raylib::prelude::*;
use glam::Vec3;

/// Update controls for a third-person camera that follows `ship_pos`.
/// - WASD moves the *ship* in 3D (W/S forward on XZ, A/D strafe)
/// - SPACE/CTRL move ship up/down
/// - Right mouse drag orbits the camera around the ship
/// - Middle mouse pans the ship (moves both ship and camera target)
/// - Mouse wheel zooms (changes orbit distance)
pub fn update_third_person(rl: &RaylibHandle, cam: &mut Camera, ship_pos: &mut Vec3, dt: f32, warp_active: bool) {
    let speed = 12.0 * dt;

    // Movement relative to camera forward (flattened to XZ)
    let mut forward = cam.forward();
    forward.y = 0.0;
    let forward = if forward.length_squared() < 1e-6 {
        Vec3::new(0.0, 0.0, -1.0)
    } else {
        forward.normalize()
    };

    let right = forward.cross(Vec3::Y).normalize();
    let up = Vec3::Y;

    // Move the ship (W/S forward/back, A/D strafe)
    if rl.is_key_down(KeyboardKey::KEY_W) {
        *ship_pos += forward * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        *ship_pos -= forward * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        *ship_pos -= right * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        *ship_pos += right * speed;
    }

    // subir / bajar
    if rl.is_key_down(KeyboardKey::KEY_SPACE) {
        *ship_pos += up * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
        *ship_pos -= up * speed;
    }

    if warp_active {
        // During warp we disable manual camera control and ship movement
        return;
    }

    // Camera orbit controls around the ship
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        let delta: Vector2 = rl.get_mouse_delta();
        let sens = 0.004; // sensibilidad
        cam.yaw -= delta.x * sens;
        cam.pitch -= delta.y * sens;
        // Clamp pitch to avoid flipping
        let limit = 1.45_f32; // ~83 degrees
        if cam.pitch > limit { cam.pitch = limit; }
        if cam.pitch < -limit { cam.pitch = -limit; }
    }

    // Zoom con rueda del ratón (acerca/aleja la cámara del target)
    let wheel = rl.get_mouse_wheel_move();
    if wheel.abs() > 0.0 {
        let delta = (-wheel) * 2.0;
        cam.distance = (cam.distance + delta).max(1.5);
    }

    // Pan (mover target/ship) con botón medio del ratón
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
        let delta: Vector2 = rl.get_mouse_delta();
        let pan_sens = 0.01 * cam.distance;
        let right_v = cam.right();
        let up_v = cam.up();
        let move_x = -delta.x * pan_sens;
        let move_y = delta.y * pan_sens;
        let shift = right_v * move_x + up_v * move_y;
        *ship_pos += shift;
    }

    // Smoothly update camera target and position to follow ship
    cam.target = *ship_pos;
    // compute desired pos from orbit spherical coords
    let cp = cam.pitch.cos();
    let x = cam.distance * cp * cam.yaw.cos();
    let z = cam.distance * cp * cam.yaw.sin();
    let y = cam.distance * cam.pitch.sin();
    let desired = cam.target + Vec3::new(x, y, z);

    // Smooth interpolation of cam.pos towards desired position
    let smooth = 1.0 - 0.85_f32.powf(dt * 60.0);
    cam.pos = cam.pos.lerp(desired, smooth);
}
