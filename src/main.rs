mod framebuffer;

use framebuffer::Framebuffer;

use raylib::prelude::*;
use nalgebra_glm as glm;
use std::f32::consts::PI;

/// Caja orientada (OBB) — centro, half-size y rotación Y (radianes)
#[derive(Clone)]
struct Cube {
    center: glm::Vec3,
    half_size: glm::Vec3,
    rot_y: f32,
    color: Color,
}

impl Cube {
    fn rotate_y(v: &glm::Vec3, angle: f32) -> glm::Vec3 {
        let ca = angle.cos();
        let sa = angle.sin();
        glm::vec3(ca * v.x + sa * v.z, v.y, -sa * v.x + ca * v.z)
    }

    fn ray_intersect(&self, ray_origin: &glm::Vec3, ray_dir: &glm::Vec3) -> Option<(f32, glm::Vec3)> {
        let local_origin = ray_origin - self.center;
        let lo = Cube::rotate_y(&local_origin, -self.rot_y);
        let ld = Cube::rotate_y(ray_dir, -self.rot_y);

        let eps = 1e-6f32;
        let bounds_min = -self.half_size;
        let bounds_max = self.half_size;

        let mut tmin = -f32::INFINITY;
        let mut tmax = f32::INFINITY;

        if ld.x.abs() < eps {
            if lo.x < bounds_min.x || lo.x > bounds_max.x { return None; }
        } else {
            let tx1 = (bounds_min.x - lo.x) / ld.x;
            let tx2 = (bounds_max.x - lo.x) / ld.x;
            let (t1, t2) = if tx1 <= tx2 { (tx1, tx2) } else { (tx2, tx1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return None; }
        }

        if ld.y.abs() < eps {
            if lo.y < bounds_min.y || lo.y > bounds_max.y { return None; }
        } else {
            let ty1 = (bounds_min.y - lo.y) / ld.y;
            let ty2 = (bounds_max.y - lo.y) / ld.y;
            let (t1, t2) = if ty1 <= ty2 { (ty1, ty2) } else { (ty2, ty1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return None; }
        }

        if ld.z.abs() < eps {
            if lo.z < bounds_min.z || lo.z > bounds_max.z { return None; }
        } else {
            let tz1 = (bounds_min.z - lo.z) / ld.z;
            let tz2 = (bounds_max.z - lo.z) / ld.z;
            let (t1, t2) = if tz1 <= tz2 { (tz1, tz2) } else { (tz2, tz1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return None; }
        }

        let t_hit = if tmin > 1e-4 { tmin } else if tmax > 1e-4 { tmax } else { return None; };

        let hit_local = lo + ld * t_hit;

        let mut normal_local = glm::vec3(0.0, 0.0, 0.0);
        if (hit_local.x - bounds_max.x).abs() < 1e-2 { normal_local = glm::vec3(1.0, 0.0, 0.0); }
        else if (hit_local.x - bounds_min.x).abs() < 1e-2 { normal_local = glm::vec3(-1.0, 0.0, 0.0); }
        else if (hit_local.y - bounds_max.y).abs() < 1e-2 { normal_local = glm::vec3(0.0, 1.0, 0.0); }
        else if (hit_local.y - bounds_min.y).abs() < 1e-2 { normal_local = glm::vec3(0.0, -1.0, 0.0); }
        else if (hit_local.z - bounds_max.z).abs() < 1e-2 { normal_local = glm::vec3(0.0, 0.0, 1.0); }
        else if (hit_local.z - bounds_min.z).abs() < 1e-2 { normal_local = glm::vec3(0.0, 0.0, -1.0); }
        else {
            let abs_pt = glm::vec3(hit_local.x.abs(), hit_local.y.abs(), hit_local.z.abs());
            if abs_pt.x > abs_pt.y && abs_pt.x > abs_pt.z {
                normal_local = glm::vec3(hit_local.x.signum(), 0.0, 0.0);
            } else if abs_pt.y > abs_pt.z {
                normal_local = glm::vec3(0.0, hit_local.y.signum(), 0.0);
            } else {
                normal_local = glm::vec3(0.0, 0.0, hit_local.z.signum());
            }
        }

        let normal_world = Cube::rotate_y(&normal_local, self.rot_y);
        let normal_world = glm::normalize(&normal_world);

        Some((t_hit, normal_world))
    }
}

fn cast_ray(cam_orig: &glm::Vec3, dir: &glm::Vec3, cube: &Cube) -> Color {
    if let Some((_t, normal)) = cube.ray_intersect(cam_orig, dir) {
        let light_dir = glm::normalize(&glm::vec3(-0.6, 0.8, -0.5));
        let diff = glm::dot(&normal, &(-light_dir)).max(0.0);
        let ambient = 0.12;
        let intensity = (ambient + diff * 0.88).min(1.0);

        let r = (cube.color.r as f32 * intensity) as u8;
        let g = (cube.color.g as f32 * intensity) as u8;
        let b = (cube.color.b as f32 * intensity) as u8;
        Color::new(r, g, b, 255)
    } else {
        let t = (dir.y + 1.0) * 0.5;
        let top = glm::vec3(40.0/255.0, 10.0/255.0, 60.0/255.0);
        let bottom = glm::vec3(90.0/255.0, 45.0/255.0, 20.0/255.0);
        let c = top * (1.0 - t) + bottom * t;
        Color::new((c.x*255.0) as u8, (c.y*255.0) as u8, (c.z*255.0) as u8, 255)
    }
}

fn render(framebuffer: &mut Framebuffer, cube: &Cube, cam_pos: &glm::Vec3, cam_yaw: f32) {
    const RENDER_SCALE: usize = 2;
    let w = (framebuffer.width() as usize / RENDER_SCALE).max(1);
    let h = (framebuffer.height() as usize / RENDER_SCALE).max(1);

    let width_f = w as f32;
    let height_f = h as f32;
    let aspect = width_f / height_f;
    let fov: f32 = 60f32.to_radians();
    let scale = (fov * 0.5).tan();

    for j in 0..h {
        for i in 0..w {
            let px = (2.0 * (i as f32 + 0.5) / width_f - 1.0) * aspect * scale;
            let py = (1.0 - 2.0 * (j as f32 + 0.5) / height_f) * scale;
            // in camera space ray_dir = normalize(px, py, -1)
            let mut ray_dir = glm::vec3(px, py, -1.0);
            ray_dir = glm::normalize(&ray_dir);
            // rotate ray_dir by cam_yaw around Y (camera orientation)
            ray_dir = Cube::rotate_y(&ray_dir, cam_yaw);

            let col = cast_ray(cam_pos, &ray_dir, cube);

            let dst_x = (i * RENDER_SCALE) as u32;
            let dst_y = (j * RENDER_SCALE) as u32;
            framebuffer.set_current_color(col);
            for oy in 0..RENDER_SCALE {
                for ox in 0..RENDER_SCALE {
                    let px = dst_x + ox as u32;
                    let py = dst_y + oy as u32;
                    if px < framebuffer.width() && py < framebuffer.height() {
                        framebuffer.set_pixel(px, py);
                    }
                }
            }
        }
    }
}

fn main() {
    const WIN_W: i32 = 900;
    const WIN_H: i32 = 900;

    let (mut rl, thread) = raylib::init().size(WIN_W, WIN_H).title("Raytrace - Mover Cámara").build();

    let mut fb = Framebuffer::new(WIN_W as u32, WIN_H as u32, Color::BLACK);

    // cámara: posición y yaw (rotación Y)
    let mut cam_pos = glm::vec3(0.0, 0.2, 0.0);
    let mut cam_yaw: f32 = 0.0;

    // cube en escena
    let cube = Cube {
        center: glm::vec3(0.0, 0.0, -3.7),
        half_size: glm::vec3(0.9, 0.9, 0.9),
        rot_y: 0.9,
        color: Color::new(200, 50, 50, 255),
    };

    // parámetros de movimiento
    let move_speed = 2.6_f32; // unidades por segundo (ajusta)
    let rot_speed = 1.6_f32; // rad/s para flechas izquierda/derecha

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // input: flechas controlan cámara
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            cam_yaw -= rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            cam_yaw += rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            // move forward in yaw direction (xz plane)
            let forward = glm::vec3(cam_yaw.cos(), 0.0, -cam_yaw.sin());
            cam_pos += forward * move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            let backward = glm::vec3(-cam_yaw.cos(), 0.0, cam_yaw.sin());
            cam_pos += backward * move_speed * dt;
        }

        fb.clear(Color::BLACK);
        render(&mut fb, &cube, &cam_pos, cam_yaw);

        fb.present(&mut rl, &thread, 1.0);
    }
}
