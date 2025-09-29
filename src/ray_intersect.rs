#![allow(dead_code)]
use nalgebra_glm::Vec3;
use raylib::prelude::Color;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub texture: Option<String>,
    pub uv_scale: f32,
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
}

impl Material {
    pub fn new_color(diffuse: Color) -> Self {
        Material { diffuse, texture: None, uv_scale: 1.0, specular: 0.0, reflectivity: 0.0, transparency: 0.0, ior: 1.0 }
    }

    pub fn with_texture(diffuse: Color, texture_path: impl Into<String>) -> Self {
        Material { diffuse, texture: Some(texture_path.into()), uv_scale: 1.0, specular: 0.0, reflectivity: 0.0, transparency: 0.0, ior: 1.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Intersect {
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: Material,
    pub normal: Vec3,
    pub point: Vec3,
    pub uv: (f32, f32),
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material, uv: (f32,f32)) -> Self {
        Intersect { distance, is_intersecting: true, material, normal, point, uv }
    }

    pub fn empty() -> Self {
        Intersect { distance: 0.0, is_intersecting: false, material: Material::new_color(Color::new(0, 0, 0, 255)), normal: Vec3::new(0.0,0.0,0.0), point: Vec3::new(0.0,0.0,0.0), uv: (0.0,0.0) }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
