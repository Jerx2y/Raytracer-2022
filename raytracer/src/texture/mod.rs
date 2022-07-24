pub mod perlin;

use image::{GenericImageView, RgbImage};

use crate::basic::vec::{Color, Point3};
use crate::texture::perlin::Perlin;

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

#[derive(Clone, Copy)]
pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new(color_value: Color) -> Self {
        Self { color_value }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.color_value
    }
}

#[derive(Clone, Copy)]
pub struct CheckerTexture<TO, TE>
where
    TO: Texture + Clone + Copy,
    TE: Texture + Clone + Copy,
{
    odd: TO,
    even: TE,
}

impl CheckerTexture<SolidColor, SolidColor> {
    #[allow(dead_code)]
    pub fn new(c1: Color, c2: Color) -> Self {
        Self {
            odd: SolidColor::new(c1),
            even: SolidColor::new(c2),
        }
    }
}

impl<TO: Texture + Clone + Copy, TE: Texture + Clone + Copy> Texture for CheckerTexture<TO, TE> {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let sines = (p.x * 10.).sin() * (p.y * 10.).sin() * (p.z * 10.).sin();
        if sines < 0. {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

#[derive(Clone, Copy)]
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        let noise = Perlin::new();
        Self { noise, scale }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        Color::new(1., 1., 1.) * 0.5 * (1. + (self.scale * p.z + 10. * self.noise.turb(p, 7)).sin())
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    width: u32,
    height: u32,
    pixel_color: Vec<[u8; 3]>,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        let img = image::open(filename).unwrap();
        let (width, height) = img.dimensions();
        let mut pixel_color: Vec<[u8; 3]> = Default::default();

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, height - y - 1);
                let tmp = [pixel[0], pixel[1], pixel[2]];
                pixel_color.push(tmp);
            }
        }

        Self {
            width,
            height,
            pixel_color,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        if self.pixel_color.is_empty() {
            return Color::new(0., 1., 1.);
        }

        let u = u.clamp(0., 1.);
        let v = v.clamp(0., 1.);

        let mut i = (u * self.width as f64) as usize;
        let mut j = (v * self.height as f64) as usize;

        if i >= self.width as usize {
            i = self.width as usize - 1;
        }

        if j >= self.height as usize {
            j = self.height as usize - 1;
        }

        let color_scale = 1. / 255.999;
        let pixel = self.pixel_color[j * self.width as usize + i];

        Color::new(
            pixel[0] as f64 * color_scale,
            pixel[1] as f64 * color_scale,
            pixel[2] as f64 * color_scale,
        )
    }
}

#[derive(Clone)]
pub struct ObjTexture {
    pub u: f64,
    pub v: f64,
    pub img: RgbImage,
}

impl ObjTexture {
    pub fn new(u: f64, v: f64, file_name: &str) -> Self {
        Self {
            u,
            v,
            img: image::open(file_name).expect("failed").to_rgb8(),
        }
    }
}

impl Texture for ObjTexture {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        let mut i = (self.u * ((self.img.width()) as f64)) as i32;
        let mut j = ((1. - self.v) * ((self.img.height()) as f64)) as i32;
        if i >= self.img.width() as i32 {
            i = self.img.width() as i32 - 1;
        }
        if j >= self.img.height() as i32 {
            j = self.img.height() as i32 - 1;
        }
        let color_scale = 1.0 / 255.999;
        let color_pixel = self.img.get_pixel(i as u32, j as u32);

        Point3::new(
            color_scale * (color_pixel[0] as f64),
            color_scale * (color_pixel[1] as f64),
            color_scale * (color_pixel[2] as f64),
        )
    }
}