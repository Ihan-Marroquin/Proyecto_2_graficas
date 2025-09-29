#![allow(dead_code)]
use crate::ray_intersect::Material;
use raylib::prelude::Color;

pub fn material_grass() -> Material {
    let mut m = Material::new_color(Color::new(80, 180, 70, 255));
    m.uv_scale = 1.0;
    m.specular = 0.08;
    m.reflectivity = 0.02;
    m
}

pub fn material_dirt() -> Material {
    let mut m = Material::new_color(Color::new(120, 85, 55, 255));
    m.uv_scale = 1.0;
    m.specular = 0.04;
    m.reflectivity = 0.00;
    m.transparency = 0.0;
    m
}

pub fn material_path() -> Material {
    let mut m = Material::new_color(Color::new(218, 187, 147, 255));
    m.uv_scale = 1.0;
    m.specular = 0.18;
    m.reflectivity = 0.06;
    m.transparency = 0.0;
    m
}

pub fn material_stone() -> Material {
    let mut m = Material::new_color(Color::new(0x66, 0x68, 0x66, 255));
    m.uv_scale = 1.0;
    m.specular = 0.25;
    m.reflectivity = 0.06;
    m.transparency = 0.0;
    m
}

pub fn material_light_gray() -> Material {
    let mut m = Material::new_color(Color::new(0xD4, 0xD3, 0xD5, 255));
    m.uv_scale = 1.0;
    m.specular = 0.25;
    m.reflectivity = 0.06;
    m.transparency = 0.0;
    m
}

pub fn material_wood() -> Material {
    let mut m = Material::new_color(Color::new(160, 115, 70, 255));
    m.uv_scale = 1.0;
    m.specular = 0.12;
    m.reflectivity = 0.03;
    m.transparency = 0.0;
    m
}

pub fn material_brick() -> Material {
    let mut m = Material::new_color(Color::new(200, 180, 145, 255));
    m.uv_scale = 1.0;
    m.specular = 0.06;
    m.reflectivity = 0.02;
    m.transparency = 0.0;
    m
}

pub fn material_water() -> Material {
    let mut m = Material::new_color(Color::new(96, 170, 230, 220));
    m.uv_scale = 1.0;
    m.specular = 0.28;
    m.reflectivity = 0.05;
    m.transparency = 0.62;
    m.ior = 1.33;
    m
}

pub fn material_glass() -> Material {
    let mut m = Material::new_color(Color::new(220, 235, 255, 150));
    m.uv_scale = 1.0;
    m.specular = 0.7;
    m.reflectivity = 0.45;
    m.transparency = 0.92;
    m.ior = 1.45;
    m
}

pub fn material_gold() -> Material {
    let mut m = Material::new_color(Color::new(255, 200, 64, 255));
    m.uv_scale = 1.0;
    m.specular = 0.7;
    m.reflectivity = 0.25;
    m.transparency = 0.0;
    m
}

pub fn material_dark_wood() -> Material {
    let mut m = Material::new_color(Color::new(0x7F, 0x66, 0x45, 255));
    m.uv_scale = 1.0;
    m.specular = 0.08;
    m.reflectivity = 0.00;
    m.transparency = 0.0;
    m
}

pub fn material_pillar() -> Material {
    let mut m = Material::new_color(Color::new(0xAF, 0x9D, 0x7B, 255));
    m.uv_scale = 1.0;
    m.specular = 0.28;
    m.reflectivity = 0.05;
    m.transparency = 0.0;
    m
}

pub fn material_pumpkin() -> Material {
    let mut m = Material::new_color(Color::new(255, 140, 48, 255));
    m.uv_scale = 1.0;
    m.specular = 0.22;
    m.reflectivity = 0.06;
    m.transparency = 0.0;
    m
}
