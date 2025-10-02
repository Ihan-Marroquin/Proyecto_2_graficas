use std::collections::HashMap;
use std::path::PathBuf;
use image::RgbaImage;
use raylib::prelude::Color;

pub struct TextureManager {
    base: PathBuf,
    images: HashMap<String, RgbaImage>,
}

impl TextureManager {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        TextureManager { base: base.into(), images: HashMap::new() }
    }

    pub fn load(&mut self, rel_path: &str) -> Result<(), String> {
        let mut p = self.base.clone();
        p.push(rel_path);
        let img = image::open(&p).map_err(|e| format!("failed to open {:?}: {}", p, e))?;
        let rgba = img.to_rgba8();
        self.images.insert(rel_path.to_string(), rgba);
        Ok(())
    }

    pub fn sample(&self, rel_path: &str, u: f32, v: f32) -> Option<Color> {
        let img = self.images.get(rel_path)?;
        if img.width() == 0 || img.height() == 0 { return None; }
        let mut uu = u.fract(); if uu < 0.0 { uu += 1.0; }
        let mut vv = v.fract(); if vv < 0.0 { vv += 1.0; }
        vv = 1.0 - vv;
        let x = (uu * (img.width() as f32)) as u32 % img.width();
        let y = (vv * (img.height() as f32)) as u32 % img.height();
        let p = img.get_pixel(x, y);
        Some(Color::new(p[0], p[1], p[2], p[3]))
    }
}
