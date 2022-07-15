mod aabb;
mod aarect;
mod bvh;
mod camera;
mod hittable;
mod material;
mod perlin;
mod ray;
mod sphere;
mod texture;
mod vec;
mod boxes;

use aarect::{XYRect, YZRect, XZRect};
use boxes::Boxes;
use camera::Camera;
use console::style;
use hittable::{Hittable, HittableList};
use image::{ImageBuffer, RgbImage};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use material::{Dielectric, DiffuseLight, Lambertian, Metal};
use rand::Rng;
use ray::Ray;
use sphere::{MovingSphere, Sphere};
use std::{
    fs::File,
    process::exit,
    sync::{mpsc, Arc},
    thread,
};
use texture::{CheckerTexture, ImageTexture, NoiseTexture};
use vec::{Color, Point3, Vec3};

use crate::bvh::BvhNode;

fn main() {
    print!("{}[2J", 27 as char); // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Set cursor position as 1,1

    // Image
    let path = "output/output.jpg";
    const IMAGE_WIDTH: u32 = 600;
    const IMAGE_HEIGHT: u32 = 600;
    const ASPECT_RATIO: f64 = IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64;
    const IMAGE_QUALITY: u8 = 100; // From 0 to 100
    const SAMPLES_PER_PIXEL: i32 = 500;
    const MAX_DEPTH: i32 = 50;
    const THREAD_NUMBER: u32 = 7;
    const SECTION_LINE_NUM: u32 = IMAGE_HEIGHT / THREAD_NUMBER;

    let vup = Vec3::new(0., 1., 0.);
    let vfov = 40.;
    let aperture = 0.0;
    let focus_dist = 10.;
    let time0 = 0.;
    let time1 = 1.;
    let lookfrom = Point3::new(278., 278., -800.);
    let lookat = Point3::new(278., 278., 0.);
    let background = Color::new(0., 0., 0.);

    println!(
        "Image size: {}\nJPEG IMAGE_QUALITY: {}",
        style(IMAGE_WIDTH.to_string() + &"x".to_string() + &IMAGE_HEIGHT.to_string()).yellow(),
        style(IMAGE_QUALITY.to_string()).yellow(),
    );

    // World
    // let main_world = random_scene();
    // let main_world = BvhNode::new_list(&random_scene(), 0., 1.);
    // let main_world = BvhNode::new_list(&two_spheres(), time0, time1);
    // let main_world = BvhNode::new_list(&two_perlin_spheres(), time0, time1);
    // let main_world = BvhNode::new_list(&earth(), time0, time1);
    // let main_world = BvhNode::new_list(&simple_light(), time0, time1);
    let main_world = BvhNode::new_list(&cornell_box(), time0, time1);

    // Camera
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        ASPECT_RATIO,
        aperture,
        focus_dist,
        time0,
        time1,
    );

    // Progress bar
    let multiprogress = Arc::new(MultiProgress::new());
    multiprogress.set_move_cursor(true);

    // Thread
    let mut output_pixel_color = Vec::<Color>::new();
    let mut thread_pool = Vec::<_>::new();

    for thread_id in 0..THREAD_NUMBER {
        // line
        let line_beg = thread_id * SECTION_LINE_NUM;
        let mut line_end = line_beg + SECTION_LINE_NUM;
        if thread_id == THREAD_NUMBER - 1 {
            line_end = IMAGE_HEIGHT;
        }

        // world
        let world = main_world.clone();

        //progress
        let mp = multiprogress.clone();
        let progress_bar = mp.add(ProgressBar::new((line_end - line_beg) as u64));
        progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] [{pos}/{len}] ({eta})")
        .progress_chars("#>-"));

        // thread code
        let (tx, rx) = mpsc::channel();

        thread_pool.push((
            thread::spawn(move || {
                let mut progress = 0;
                progress_bar.set_position(progress);

                let mut section_pixel_color = Vec::<Color>::new();

                let mut rng = rand::thread_rng();
                for y in line_beg..line_end {
                    for x in 0..IMAGE_WIDTH {
                        let mut pixel_color = Color::new(0., 0., 0.);
                        for _i in 0..SAMPLES_PER_PIXEL {
                            let rand_u: f64 = rng.gen();
                            let rand_v: f64 = rng.gen();
                            let u = (x as f64 + rand_u) / (IMAGE_WIDTH - 1) as f64;
                            let v = (y as f64 + rand_v) / (IMAGE_HEIGHT - 1) as f64;
                            let r = cam.get_ray(u, v);
                            pixel_color += ray_color(r, background, &world, MAX_DEPTH);
                        }
                        section_pixel_color.push(pixel_color);
                    }
                    progress += 1;
                    progress_bar.set_position(progress);
                }
                tx.send(section_pixel_color).unwrap();
                progress_bar.finish_with_message("Finished.");
            }),
            rx,
        ));
    }
    multiprogress.join().unwrap();

    let mut thread_finish = true;
    for _thread_id in 0..THREAD_NUMBER {
        let thread = thread_pool.remove(0);
        match thread.0.join() {
            Ok(_) => {
                let mut received = thread.1.recv().unwrap();
                output_pixel_color.append(&mut received);
            }
            Err(_) => {
                thread_finish = false;
            }
        }
    }

    if !thread_finish {
        println!("run time error");
        exit(0);
    }

    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let mut pixel_id = 0;
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let pixel_color = output_pixel_color[pixel_id];
            let pixel = img.get_pixel_mut(x, IMAGE_HEIGHT - y - 1);
            *pixel = image::Rgb(write_color(pixel_color, SAMPLES_PER_PIXEL));
            pixel_id += 1;
        }
    }

    // ==================== afterwork ====================

    // Output image to file
    println!("Ouput image as \"{}\"", style(path).yellow());
    let output_image = image::DynamicImage::ImageRgb8(img);
    let mut output_file = File::create(path).unwrap();
    match output_image.write_to(
        &mut output_file,
        image::ImageOutputFormat::Jpeg(IMAGE_QUALITY),
    ) {
        Ok(_) => {}
        // Err(_) => panic!("Outputting image fails."),
        Err(_) => println!("{}", style("Outputting image fails.").red()),
    }

    exit(0);
}

fn ray_color(r: Ray, background: Color, world: &BvhNode, depth: i32) -> Color {
    if depth <= 0 {
        return Color::new(0., 0., 0.);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::MAX) {
        let emitted = rec.mat_ptr.emitted(rec.u, rec.v, rec.p);
        if let Some((attenuation, scattered)) = rec.mat_ptr.scatter(r, &rec) {
            emitted + attenuation * ray_color(scattered, background, world, depth - 1)
        } else {
            emitted
        }
    } else {
        background
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

#[allow(dead_code)]
fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let checker = Arc::new(CheckerTexture::new(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Arc::new(Lambertian::new_arc(checker)),
    )));

    //    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    //    world.add(Arc::new(Sphere::new(
    //        Point3::new(0., -1000., 0.),
    //        1000.,
    //        ground_material,
    //    )));

    let mut rng = rand::thread_rng();
    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );

            if (center - Point3::new(4., 0.2, 0.)).length() > 0.9 {
                if choose_mat < 0.80 {
                    let albedo = Color::random(0., 1.);
                    let center2 = center + Vec3::new(0., rng.gen_range(0.0..0.5), 0.);
                    world.add(Arc::new(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        1.0,
                        0.2,
                        Arc::new(Lambertian::new(albedo)),
                    )));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.);
                    let fuzz = rng.gen_range(0.0..0.5);
                    world.add(Arc::new(Sphere::new(
                        center,
                        0.2,
                        Arc::new(Metal::new(albedo, fuzz)),
                    )));
                } else {
                    world.add(Arc::new(Sphere::new(
                        center,
                        0.2,
                        Arc::new(Dielectric::new(1.5)),
                    )));
                }
            }
        }
    }

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 1., 0.),
        1.,
        Arc::new(Dielectric::new(1.5)),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(-4., 1., 0.),
        1.,
        Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1))),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(4., 1., 0.),
        1.,
        Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.)),
    )));

    world
}

#[allow(dead_code)]
fn two_spheres() -> HittableList {
    let mut world = HittableList::new();

    let checker = Arc::new(CheckerTexture::new(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., -10., 0.),
        10.,
        Arc::new(Lambertian::new_arc(checker.clone())),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 10., 0.),
        10.,
        Arc::new(Lambertian::new_arc(checker)),
    )));

    world
}

#[allow(dead_code)]
fn two_perlin_spheres() -> HittableList {
    let mut world = HittableList::new();
    let pertext = Arc::new(NoiseTexture::new(4.));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Arc::new(Lambertian::new_arc(pertext.clone())),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 2., 0.),
        2.,
        Arc::new(Lambertian::new_arc(pertext)),
    )));

    world
}

#[allow(dead_code)]
fn earth() -> HittableList {
    let earth_texture = Arc::new(ImageTexture::new("input/earthmap.jpg"));
    let earth_surface = Arc::new(Lambertian::new_arc(earth_texture));

    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 0., 0.),
        2.,
        earth_surface,
    )));

    world
}

#[allow(dead_code)]
fn simple_light() -> HittableList {
    let mut world = HittableList::new();
    let pertext = Arc::new(NoiseTexture::new(4.));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Arc::new(Lambertian::new_arc(pertext.clone())),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 2., 0.),
        2.,
        Arc::new(Lambertian::new_arc(pertext)),
    )));

    let difflight = Arc::new(DiffuseLight::new(Color::new(4., 4., 4.)));
    world.add(Arc::new(XYRect::new(3., 5., 1., 3., -2., difflight)));

    world
}

#[allow(dead_code)]
fn cornell_box() -> HittableList {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(15., 15., 15.)));

    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 555., green)));
    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 0., red)));
    world.add(Arc::new(XZRect::new(213., 343., 227., 332., 554., light)));
    world.add(Arc::new(XZRect::new(0., 555., 0., 555., 0., white.clone())));
    world.add(Arc::new(XZRect::new(0., 555., 0., 555., 555., white.clone())));
    world.add(Arc::new(XYRect::new(0., 555., 0., 555., 555., white.clone())));

    world.add(Arc::new(Boxes::new(Point3::new(130., 0., 65.), Point3::new(295., 165., 230.), white.clone())));
    world.add(Arc::new(Boxes::new(Point3::new(265., 0., 295.), Point3::new(430., 330., 460.), white)));

    world
}