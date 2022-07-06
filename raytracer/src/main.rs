use std::{fs::File, process::exit};

use image::{ImageBuffer, RgbImage};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

mod vec;
use vec::{Color, Point3, Vec3};

mod ray;
use ray::Ray;

fn main() {
    print!("{}[2J", 27 as char); // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Set cursor position as 1,1

    let width = 400;
    let height = 225;
    let aspect_ratio = width as f64 / height as f64;
    let quality = 100; // From 0 to 100
    let path = "output/output.jpg";

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

    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    for y in 0..height {
        for x in 0..width {
            let u = x as f64 / (width - 1) as f64;
            let v = y as f64 / (height - 1) as f64;
            let r = Ray::new(
                origin,
                lower_left_corner + horizontal * u + vertical * v - origin,
            );
            let pixel_color: Color = ray_color(r);
            let pixel = img.get_pixel_mut(x, height - y - 1);
            *pixel = image::Rgb(to_color256(pixel_color));
            progress.inc(1);
        }
    }
    progress.finish();

    // ==================== afterwork ====================

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

fn ray_color(r: Ray) -> Color {
    if hit_sphere(Point3::new(0., 0., -1.), 0.5, r) {
        return Color::new(1., 0., 0.);
    }
    let unit_direction = r.dir.to_unit();
    let t = 0.5 * (unit_direction.y + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

fn to_color256(c: Color) -> [u8; 3] {
    [
        (c.x * 255.999).floor() as u8,
        (c.y * 255.999).floor() as u8,
        (c.z * 255.999).floor() as u8,
    ]
}

fn hit_sphere(center: Point3, radius: f64, r: Ray) -> bool {
    let oc = r.orig - center;
    let a = Vec3::dot(r.dir, r.dir);
    let b = 2.0 * Vec3::dot(oc, r.dir);
    let c = Vec3::dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4. * a * c;
    discriminant > 0.
}
