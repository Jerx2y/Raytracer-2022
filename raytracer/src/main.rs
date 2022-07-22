mod basic;
mod hittable;
mod material;
mod scene;
mod texture;

use console::style;
use image::{ImageBuffer, RgbImage};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::Rng;
use std::{
    fs::File,
    process::exit,
    sync::{mpsc, Arc},
    thread,
    time::Instant,
};

use basic::{camera::Camera, pdf::Pdf};
use basic::{pdf::HittablePdf, ray::Ray};
use basic::{
    pdf::MixturePdf,
    vec::{Color, Point3, Vec3},
};
use hittable::bvh::BvhNode;
use hittable::{Hittable, HittableList};

fn main() {
    print!("{}[2J", 27 as char); // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Set cursor position as 1,1

    // Image
    let path = "output/output.jpg";
    const IMAGE_WIDTH: u32 = 500;
    const IMAGE_HEIGHT: u32 = 500;
    const ASPECT_RATIO: f64 = IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64;
    const IMAGE_QUALITY: u8 = 100; // From 0 to 100
    const SAMPLES_PER_PIXEL: i32 = 100;
    const MAX_DEPTH: i32 = 50;
    const THREAD_NUMBER: u32 = 8;
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

    let begin_time = Instant::now();
    println!(
        "{} 💿 {}",
        style("[1/5]").bold().dim(),
        style("Initlizing...").green()
    );
    println!(
        "IMAGE SIZE: {}\nJPEG QUALITY: {}\nSAMPLE PER PIXEL: {}\nMAX DEPTH: {}",
        style(IMAGE_WIDTH.to_string() + &"x".to_string() + &IMAGE_HEIGHT.to_string()).yellow(),
        style(IMAGE_QUALITY.to_string()).yellow(),
        style(SAMPLES_PER_PIXEL.to_string()).yellow(),
        style(MAX_DEPTH.to_string()).yellow(),
    );

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

    println!(
        "{} 🚀 {} {} {}",
        style("[2/5]").bold().dim(),
        style("Rendering with").green(),
        style(THREAD_NUMBER.to_string()).yellow(),
        style("Threads...").green(),
    );

    // World & lights
    let (main_world, main_lights) = scene::cornell_box();
    let main_world = BvhNode::new_list(&main_world, time0, time1);

    // Random line
    let mut random_line_id: [u32; IMAGE_HEIGHT as usize] = [0; IMAGE_HEIGHT as usize];
    let mut rng = rand::thread_rng();
    for i in 0..IMAGE_HEIGHT {
        random_line_id[i as usize] = i;
        let target = rng.gen_range(0..i + 1);
        random_line_id.swap(i as usize, target as usize);
    }

    // Progress bar
    let multiprogress = Arc::new(MultiProgress::new());
    multiprogress.set_move_cursor(true);

    // Thread
    let mut output_pixel_color = Vec::<Color>::new();
    let mut thread_pool = Vec::<_>::new();

    for thread_id in 0..THREAD_NUMBER {
        // line
        let line_id = random_line_id;
        let line_beg = thread_id * SECTION_LINE_NUM;
        let mut line_end = line_beg + SECTION_LINE_NUM;
        if thread_id == THREAD_NUMBER - 1 {
            line_end = IMAGE_HEIGHT;
        }

        // world & lights
        let world = main_world.clone();
        let lights = main_lights.clone();

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
                for y_id in line_beg..line_end {
                    for x in 0..IMAGE_WIDTH {
                        let y = line_id[y_id as usize];
                        let mut pixel_color = Color::new(0., 0., 0.);
                        for _i in 0..SAMPLES_PER_PIXEL {
                            let rand_u: f64 = rng.gen();
                            let rand_v: f64 = rng.gen();
                            let u = (x as f64 + rand_u) / (IMAGE_WIDTH - 1) as f64;
                            let v = (y as f64 + rand_v) / (IMAGE_HEIGHT - 1) as f64;
                            let r = cam.get_ray(u, v);
                            pixel_color += ray_color(r, background, &world, &lights, MAX_DEPTH);
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

    println!(
        "{} 🚛 {}",
        style("[3/5]").bold().dim(),
        style("Collecting Threads Results...").green(),
    );

    for _thread_id in 0..THREAD_NUMBER {
        let thread = thread_pool.remove(0);
        match thread.0.join() {
            Ok(_) => {
                let mut received = thread.1.recv().unwrap();
                output_pixel_color.append(&mut received);
            }
            Err(_) => {
                println!("Thread error");
                exit(0);
            }
        }
    }

    println!(
        "{} 🏭 {}",
        style("[4/5]").bold().dim(),
        style("Generating Image...").green()
    );

    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let mut pixel_id = 0;
    for y_id in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let y = random_line_id[y_id as usize];
            let pixel_color = output_pixel_color[pixel_id];
            let pixel = img.get_pixel_mut(x, IMAGE_HEIGHT - y - 1);
            *pixel = image::Rgb(write_color(pixel_color, SAMPLES_PER_PIXEL));
            pixel_id += 1;
        }
    }

    println!(
        "{} 🥽 {}",
        style("[5/5]").bold().dim(),
        style("Outping Image...").green()
    );

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
        Err(_) => println!("{}", style("Outputting image fails.").red()),
    }

    println!(
        "\n      🎉 {}\n      🕒 Elapsed Time: {}",
        style("All Work Done.").bold().green(),
        style(HumanDuration(begin_time.elapsed())).yellow(),
    );
    println!("\n");

    exit(0);
}

fn ray_color(
    r: Ray,
    background: Color,
    world: &BvhNode,
    lights: &HittableList,
    depth: i32,
) -> Color {
    if depth <= 0 {
        return Color::new(0., 0., 0.);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::MAX) {
        let emitted = rec.mat_ptr.emitted(r, &rec, rec.u, rec.v, rec.p);
        if let Some(srec) = rec.mat_ptr.scatter(r, &rec) {
            if let Some(specular) = srec.specular_ray {
                return srec.attenuation
                    * ray_color(specular, background, &world, lights, depth - 1);
            }

            // if srec.pdf_ptr.is_none() {
            //     return emitted
            //         + srec.attenuation
            //             * ray_color(
            //                 srec.specular_ray.unwrap(),
            //                 background,
            //                 &world,
            //                 lights,
            //                 depth - 1,
            //             );
            // }

            let light_ptr = HittablePdf::new(lights, rec.p);
            let p = MixturePdf::new(light_ptr, srec.pdf_ptr.unwrap());
            let scattered = Ray::new(rec.p, p.generate(), r.tm);
            let pdf_val = p.value(scattered.dir);
            emitted
                + srec.attenuation
                    * rec.mat_ptr.scattering_pdf(r, &rec, scattered)
                    * ray_color(scattered, background, &world, lights, depth - 1)
                    / pdf_val
        } else {
            emitted
        }
    } else {
        background
    }
}

fn write_color(pixel_color: Color, samples_per_pixel: i32) -> [u8; 3] {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b = pixel_color.z;
    if r.is_nan() {
        r = 0.
    }
    if g.is_nan() {
        g = 0.
    }
    if b.is_nan() {
        b = 0.
    }

    [
        ((r / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999) * 255.999).floor() as u8,
        ((g / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999) * 255.999).floor() as u8,
        ((b / samples_per_pixel as f64).sqrt().clamp(0.0, 0.999) * 255.999).floor() as u8,
    ]
}
