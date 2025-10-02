#![allow(dead_code)]
use nalgebra_glm as glm;

use crate::ray_intersect::{Material, Intersect, RayIntersect, FaceId};

#[derive(Clone)]
pub struct Cube {
    pub center: glm::Vec3,
    pub half_size: glm::Vec3,
    pub rot_y: f32,
    pub material: Material,
    pub top_material: Option<Material>,
    pub radius: f32,
}

impl Cube {
    pub fn rotate_y(v: &glm::Vec3, angle: f32) -> glm::Vec3 {
        let ca = angle.cos();
        let sa = angle.sin();
        glm::vec3(ca * v.x + sa * v.z, v.y, -sa * v.x + ca * v.z)
    }
    pub fn rotate_x(v: &glm::Vec3, angle: f32) -> glm::Vec3 {
        let ca = angle.cos();
        let sa = angle.sin();
        glm::vec3(v.x, ca * v.y - sa * v.z, sa * v.y + ca * v.z)
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &glm::Vec3, ray_dir: &glm::Vec3) -> Intersect {
        let to_center = self.center - *ray_origin;
        let proj = glm::dot(&to_center, ray_dir).max(0.0);
        let closest = *ray_origin + ray_dir * proj;
        let dist2 = glm::distance2(&closest, &self.center);
        if dist2 > self.radius * self.radius {
            return Intersect::empty();
        }

        let local_origin = ray_origin - self.center;
        let lo = Cube::rotate_y(&local_origin, -self.rot_y);
        let ld = Cube::rotate_y(ray_dir, -self.rot_y);

        let eps = 1e-6f32;
        let bounds_min = -self.half_size;
        let bounds_max = self.half_size;

        let mut tmin = -f32::INFINITY;
        let mut tmax = f32::INFINITY;

        if ld.x.abs() < eps {
            if lo.x < bounds_min.x || lo.x > bounds_max.x { return Intersect::empty(); }
        } else {
            let tx1 = (bounds_min.x - lo.x) / ld.x;
            let tx2 = (bounds_max.x - lo.x) / ld.x;
            let (t1, t2) = if tx1 <= tx2 { (tx1, tx2) } else { (tx2, tx1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return Intersect::empty(); }
        }

        if ld.y.abs() < eps {
            if lo.y < bounds_min.y || lo.y > bounds_max.y { return Intersect::empty(); }
        } else {
            let ty1 = (bounds_min.y - lo.y) / ld.y;
            let ty2 = (bounds_max.y - lo.y) / ld.y;
            let (t1, t2) = if ty1 <= ty2 { (ty1, ty2) } else { (ty2, ty1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return Intersect::empty(); }
        }

        if ld.z.abs() < eps {
            if lo.z < bounds_min.z || lo.z > bounds_max.z { return Intersect::empty(); }
        } else {
            let tz1 = (bounds_min.z - lo.z) / ld.z;
            let tz2 = (bounds_max.z - lo.z) / ld.z;
            let (t1, t2) = if tz1 <= tz2 { (tz1, tz2) } else { (tz2, tz1) };
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmax < tmin { return Intersect::empty(); }
        }

        let t_hit = if tmin > 1e-4 { tmin } else if tmax > 1e-4 { tmax } else { return Intersect::empty(); };

        let hit_local = lo + ld * t_hit;

        let normal_local = if (hit_local.x - bounds_max.x).abs() < 1e-2 {
            glm::vec3(1.0, 0.0, 0.0)
        } else if (hit_local.x - bounds_min.x).abs() < 1e-2 {
            glm::vec3(-1.0, 0.0, 0.0)
        } else if (hit_local.y - bounds_max.y).abs() < 1e-2 {
            glm::vec3(0.0, 1.0, 0.0)
        } else if (hit_local.y - bounds_min.y).abs() < 1e-2 {
            glm::vec3(0.0, -1.0, 0.0)
        } else if (hit_local.z - bounds_max.z).abs() < 1e-2 {
            glm::vec3(0.0, 0.0, 1.0)
        } else if (hit_local.z - bounds_min.z).abs() < 1e-2 {
            glm::vec3(0.0, 0.0, -1.0)
        } else {
            let abs_pt = glm::vec3(hit_local.x.abs(), hit_local.y.abs(), hit_local.z.abs());
            if abs_pt.x > abs_pt.y && abs_pt.x > abs_pt.z {
                glm::vec3(hit_local.x.signum(), 0.0, 0.0)
            } else if abs_pt.y > abs_pt.z {
                glm::vec3(0.0, hit_local.y.signum(), 0.0)
            } else {
                glm::vec3(0.0, 0.0, hit_local.z.signum())
            }
        };

        let normal_world = Cube::rotate_y(&normal_local, self.rot_y);
    let normal_world = glm::normalize(&normal_world);

        let hit_world = *ray_origin + *ray_dir * t_hit;

        let (u, v) = if normal_local.x.abs() > 0.5 {
            let mut uu = (hit_local.z - bounds_min.z) / (bounds_max.z - bounds_min.z);
            let vv = (hit_local.y - bounds_min.y) / (bounds_max.y - bounds_min.y);
            if normal_local.x > 0.0 { uu = 1.0 - uu; }
            (uu, vv)
        } else if normal_local.y.abs() > 0.5 {
            let mut uu = (hit_local.x - bounds_min.x) / (bounds_max.x - bounds_min.x);
            let mut vv = (hit_local.z - bounds_min.z) / (bounds_max.z - bounds_min.z);
            if normal_local.y > 0.0 {
                let (a, b) = (uu, vv);
                uu = b;
                vv = 1.0 - a;
            } else {
                vv = 1.0 - vv;
            }
            (uu, vv)
        } else {
            let mut uu = (hit_local.x - bounds_min.x) / (bounds_max.x - bounds_min.x);
            let vv = (hit_local.y - bounds_min.y) / (bounds_max.y - bounds_min.y);
            if normal_local.z < 0.0 { uu = 1.0 - uu; }
            (uu, vv)
        };

        let mut mat = self.material.clone();
        if normal_local.y.abs() > 0.5 && normal_local.y > 0.0 {
            if let Some(ref top) = self.top_material {
                mat = top.clone();
            }
        }

        let face = if normal_local.x.abs() > 0.5 {
            if normal_local.x > 0.0 { FaceId::Right } else { FaceId::Left }
        } else if normal_local.y.abs() > 0.5 {
            if normal_local.y > 0.0 { FaceId::Top } else { FaceId::Bottom }
        } else if normal_local.z.abs() > 0.5 {
            if normal_local.z > 0.0 { FaceId::Front } else { FaceId::Back }
        } else { FaceId::Unknown };

        Intersect::new(hit_world, normal_world, t_hit, mat, (u, v), face)
    }
}
