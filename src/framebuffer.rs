use raylib::prelude::*;
// imports relacionados con escritura de archivos fueron removidos

// Wrapper sencillo de framebuffer sobre raylib Image
pub struct Framebuffer {
    width: u32,
    height: u32,
    color_buffer: Image,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background);
        Framebuffer {
            width,
            height,
            color_buffer,
            current_color: Color::WHITE,
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    pub fn clear(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, color);
            }
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            Image::draw_pixel(&mut self.color_buffer, x as i32, y as i32, self.current_color);
        }
    }

    pub fn present(&self, window: &mut RaylibHandle, thread: &RaylibThread, scale: f32) {
        if let Ok(texture) = window.load_texture_from_image(thread, &self.color_buffer) {
            let mut d = window.begin_drawing(thread);
            d.draw_texture_ex(&texture, Vector2::new(0.0, 0.0), 0.0, scale, Color::WHITE);
        }
    }

    // write_png eliminado intencionalmente: comportamiento de captura removido.

    #[allow(dead_code)]
    pub fn image(&self) -> &Image { &self.color_buffer }
}
