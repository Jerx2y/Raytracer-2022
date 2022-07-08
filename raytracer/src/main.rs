mod camera;
mod hittable;
mod ray;
mod sphere;
mod vec;

use camera::Camera;
use console::style;
use hittable::{/* HitRecord,*/ Hittable, HittableList};
use image::{ImageBuffer, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use ray::Ray;
use sphere::Sphere;
use std::{fs::File, process::exit};
use vec::{Color, Point3, Vec3};

fn main() {
    print!("{}[2J", 27 as char); // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Set cursor position as 1,1

    // Image
    let width = 400;
    let height = 225;
    let aspect_ratio = width as f64 / height as f64;
    let quality = 100; // From 0 to 100
    let path = "output/output.jpg";
    let samples_per_pixel = 100;
    let max_depth = 50;

    println!(
        "Image size: {}\nJPEG quality: {}",
        style(width.to_string() + &"x".to_string() + &height.to_string()).yellow(),
        style(quality.to_string()).yellow(),
    );

    // Create image data
    let mut img: RgbImage = ImageBuffer::new(width, height);
    // Progress bar UI powered by library `indicatif`
    // Get environment variable CI, which is true for GitHub Action
    let progress = if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new((height * width) as u64)
    };
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] [{pos}/{len}] ({eta})")
        .progress_chars("#>-"));

    // ===================== prework =====================

    // Generate image

    // World
    let mut world: HittableList = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0., 0., -1.), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0., -100.5, -1.), 100.)));

    // Camera
    let cam = Camera::new(aspect_ratio);

    let mut rng = rand::thread_rng();
    for y in 0..height {
        for x in 0..width {
            let mut pixel_color = Color::new(0., 0., 0.);
            for _i in 0..samples_per_pixel {
                let rand_u: f64 = rng.gen();
                let rand_v: f64 = rng.gen();
                let u = (x as f64 + rand_u) / (width - 1) as f64;
                let v = (y as f64 + rand_v) / (height - 1) as f64;
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(r, &world, max_depth);
            }
            let pixel = img.get_pixel_mut(x, height - y - 1);
            *pixel = image::Rgb(write_color(pixel_color, samples_per_pixel));
            progress.inc(1);
        }
    }

    // ==================== afterwork ====================

    progress.finish();
    // Output image to file
    println!("Ouput image as \"{}\"", style(path).yellow());
    let output_image = image::DynamicImage::ImageRgb8(img);
    let mut output_file = File::create(path).unwrap();
    match output_image.write_to(&mut output_file, image::ImageOutputFormat::Jpeg(quality)) {
        Ok(_) => {}
        // Err(_) => panic!("Outputting image fails."),
        Err(_) => println!("{}", style("Outputting image fails.").red()),
    }

    exit(0);
}

fn ray_color(r: Ray, world: &HittableList, depth: i32) -> Color {
    if depth <= 0 {
        return Color::new(0., 0., 0.);
    }
    if let Some(rec) = world.hit(r, 0., f64::MAX) {
        let target = rec.p + rec.normal + Vec3::random_in_unit_sphere();
        ray_color(Ray::new(rec.p, target - rec.p), world, depth - 1) * 0.5
    } else {
        // background
        let unit_direction = r.dir.to_unit();
        let t = 0.5 * (unit_direction.y + 1.0);
        Color::new(1., 1., 1.) * (1. - t) + Color::new(0.5, 0.7, 1.) * t
    }
}

fn write_color(pixel_color: Color, samples_per_pixel: i32) -> [u8; 3] {
    [
        ((pixel_color.x / samples_per_pixel as f64)
            .sqrt()
            .clamp(0.0, 0.999)
            * 255.999)
            .floor() as u8,
        ((pixel_color.y / samples_per_pixel as f64)
            .sqrt()
            .clamp(0.0, 0.999)
            * 255.999)
            .floor() as u8,
        ((pixel_color.z / samples_per_pixel as f64)
            .sqrt()
            .clamp(0.0, 0.999)
            * 255.999)
            .floor() as u8,
    ]
}

/*
fn hit_sphere(center: Point3, radius: f64, r: Ray) -> f64 {
    let oc = r.orig - center;
    let a = Vec3::dot(r.dir, r.dir);
    let b = 2.0 * Vec3::dot(oc, r.dir);
    let c = Vec3::dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4. * a * c;
    if discriminant < 0. {
        -1.
    } else {
        (-b - discriminant.sqrt()) / (2. * a)
    }
}
*/
