mod basic;
mod hittable;
mod material;
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
use hittable::boxes::Boxes;
use hittable::bvh::BvhNode;
use hittable::constantmedium::ConstantMedium;
use hittable::sphere::{MovingSphere, Sphere};
use hittable::{
    aarect::{XYRect, XZRect, YZRect},
    FlipFace,
};
use hittable::{Hittable, HittableList, RotateY, Translate};
use material::{Dielectric, DiffuseLight, Lambertian, Metal};
use texture::{CheckerTexture, ImageTexture, NoiseTexture};

fn main() {
    print!("{}[2J", 27 as char); // Clear screen
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Set cursor position as 1,1

    let begin_time = Instant::now();
    println!(
        "{} 💿 {}",
        style("[1/5]").bold().dim(),
        style("Initlizing...").green()
    );

    // Image
    let path = "output/output.jpg";
    const IMAGE_WIDTH: u32 = 600;
    const IMAGE_HEIGHT: u32 = 600;
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

    println!(
        "IMAGE SIZE: {}\nJPEG QUALITY: {}\nSAMPLE PER PIXEL: {}\nMAX DEPTH: {}",
        style(IMAGE_WIDTH.to_string() + &"x".to_string() + &IMAGE_HEIGHT.to_string()).yellow(),
        style(IMAGE_QUALITY.to_string()).yellow(),
        style(SAMPLES_PER_PIXEL.to_string()).yellow(),
        style(MAX_DEPTH.to_string()).yellow(),
    );

    // World
    // let main_world = random_scene();
    // let main_world = BvhNode::new_list(&random_scene(), 0., 1.);
    // let main_world = BvhNode::new_list(&two_spheres(), time0, time1);
    // let main_world = BvhNode::new_list(&two_perlin_spheres(), time0, time1);
    // let main_world = BvhNode::new_list(&earth(), time0, time1);
    // let main_world = BvhNode::new_list(&simple_light(), time0, time1);
    let main_world = BvhNode::new_list(&cornell_box(), time0, time1);
    // let main_world = BvhNode::new_list(&cornell_smoke(), time0, time1);
    // let main_world = BvhNode::new_list(&final_scene(), time0, time1);

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

        // world
        let world = main_world.clone();
        // let lights = Arc::new(XZRect::new(
        //     213.,
        //     343.,
        //     227.,
        //     332.,
        //     554.,
        //     Arc::new(Dielectric::new(0.)),
        // ));
        let lights = Arc::new(Sphere::new(
            Point3::new(190., 90., 190.),
            90.,
            Arc::new(Dielectric::new(0.)),
        ));


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
                            pixel_color +=
                                ray_color(r, background, &world, lights.clone(), MAX_DEPTH);
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
        // Err(_) => panic!("Outputting image fails."),
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
    lights: Arc<dyn Hittable>,
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
                    * ray_color(specular, background, world, lights, depth - 1);
            }

            let light_ptr = Arc::new(HittablePdf::new(lights.clone(), rec.p));
            let p = MixturePdf::new(light_ptr, srec.pdf_ptr.unwrap());
            let scattered = Ray::new(rec.p, p.generate(), r.tm);
            let mut pdf_val = p.value(scattered.dir);
            if pdf_val <= 0. {
                pdf_val = 1.;
            }
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
    let mut world: HittableList = Default::default();

    let checker = Arc::new(CheckerTexture::new(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Arc::new(Lambertian::new_arc(checker)),
    )));

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
    let mut world: HittableList = Default::default();

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
    let mut world: HittableList = Default::default();
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

    let mut world: HittableList = Default::default();

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 0., 0.),
        2.,
        earth_surface,
    )));

    world
}

#[allow(dead_code)]
fn simple_light() -> HittableList {
    let mut world: HittableList = Default::default();
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
    let mut world: HittableList = Default::default();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(15., 15., 15.)));

    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 555., green)));
    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 0., red)));
    world.add(Arc::new(FlipFace::new(Arc::new(XZRect::new(
        213., 343., 227., 332., 554., light,
    )))));
    world.add(Arc::new(XZRect::new(0., 555., 0., 555., 0., white.clone())));
    world.add(Arc::new(XZRect::new(
        0.,
        555.,
        0.,
        555.,
        555.,
        white.clone(),
    )));
    world.add(Arc::new(XYRect::new(
        0.,
        555.,
        0.,
        555.,
        555.,
        white.clone(),
    )));

    let aluminum = Arc::new(Metal::new(Color::new(0.8, 0.85, 0.88), 0.));
    let mut box1: Arc<dyn Hittable> = Arc::new(Boxes::new(
        Point3::new(0., 0., 0.),
        Point3::new(165., 330., 165.),
        aluminum,
    ));
    box1 = Arc::new(RotateY::new(box1, 15.));
    box1 = Arc::new(Translate::new(box1, Vec3::new(265., 0., 295.)));
    world.add(box1);

    let glass = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(
        Point3::new(190., 90., 190.),
        90.,
        glass,
    )));

    // let mut box2: Arc<dyn Hittable> = Arc::new(Boxes::new(
    //     Point3::new(0., 0., 0.),
    //     Point3::new(165., 165., 165.),
    //     white,
    // ));
    // box2 = Arc::new(RotateY::new(box2, -18.));
    // box2 = Arc::new(Translate::new(box2, Vec3::new(130., 0., 65.)));
    // world.add(box2);

    world
}

#[allow(dead_code)]
pub fn cornell_smoke() -> HittableList {
    let mut world: HittableList = Default::default();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(7., 7., 7.)));

    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 555., green)));
    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 0., red)));
    world.add(Arc::new(XZRect::new(113., 443., 127., 432., 554., light)));
    world.add(Arc::new(XZRect::new(
        0.,
        555.,
        0.,
        555.,
        555.,
        white.clone(),
    )));
    world.add(Arc::new(XZRect::new(0., 555., 0., 555., 0., white.clone())));
    world.add(Arc::new(XYRect::new(
        0.,
        555.,
        0.,
        555.,
        555.,
        white.clone(),
    )));

    let mut box1: Arc<dyn Hittable> = Arc::new(Boxes::new(
        Point3::new(0., 0., 0.),
        Point3::new(165., 330., 165.),
        white.clone(),
    ));
    box1 = Arc::new(RotateY::new(box1, 15.));
    box1 = Arc::new(Translate::new(box1, Vec3::new(265., 0., 295.)));
    world.add(Arc::new(ConstantMedium::new(
        box1,
        0.01,
        Color::new(0., 0., 0.),
    )));

    let mut box2: Arc<dyn Hittable> = Arc::new(Boxes::new(
        Point3::new(0., 0., 0.),
        Point3::new(165., 165., 165.),
        white,
    ));
    box2 = Arc::new(RotateY::new(box2, -18.));
    box2 = Arc::new(Translate::new(box2, Vec3::new(130., 0., 65.)));
    world.add(Arc::new(ConstantMedium::new(
        box2,
        0.01,
        Color::new(1., 1., 1.),
    )));

    world
}

#[allow(dead_code)]
pub fn final_scene() -> HittableList {
    let mut box1: HittableList = Default::default();
    let ground = Arc::new(Lambertian::new(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    let mut rng = rand::thread_rng();
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.;
            let x0 = -1000. + i as f64 * w;
            let z0 = -1000. + j as f64 * w;
            let y0 = 0.;
            let x1 = x0 + w;
            let y1 = rng.gen_range(1.0..101.0);
            let z1 = z0 + w;

            box1.add(Arc::new(Boxes::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    let mut world: HittableList = Default::default();

    world.add(Arc::new(BvhNode::new_list(&box1, 0., 1.)));

    let light = Arc::new(DiffuseLight::new(Color::new(7., 7., 7.)));
    world.add(Arc::new(XZRect::new(123., 423., 147., 412., 554., light)));

    let center1 = Point3::new(400., 400., 200.);
    let center2 = center1 + Vec3::new(25., 0., 0.);
    let moving_sphere_material = Arc::new(Lambertian::new(Color::new(0.7, 0.3, 0.1)));
    world.add(Arc::new(MovingSphere::new(
        center1,
        center2,
        0.,
        1.,
        50.,
        moving_sphere_material,
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(260., 150., 45.),
        50.,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., 150., 145.),
        50.,
        Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.)),
    )));

    let boundary = Arc::new(Sphere::new(
        Point3::new(360., 150., 145.),
        70.,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(boundary.clone());
    world.add(Arc::new(ConstantMedium::new(
        boundary,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));
    let boundary = Arc::new(Sphere::new(
        Point3::new(0., 0., 0.),
        5000.,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(Arc::new(ConstantMedium::new(
        boundary,
        0.0001,
        Color::new(1., 1., 1.),
    )));

    let emat = Arc::new(Lambertian::new_arc(Arc::new(ImageTexture::new(
        "input/earthmap.jpg",
    ))));
    world.add(Arc::new(Sphere::new(
        Point3::new(400., 200., 400.),
        100.,
        emat,
    )));

    let pertext = Arc::new(NoiseTexture::new(0.1));
    world.add(Arc::new(Sphere::new(
        Point3::new(220., 280., 300.),
        80.,
        Arc::new(Lambertian::new_arc(pertext)),
    )));

    let mut box2: HittableList = Default::default();
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _i in 0..ns {
        box2.add(Arc::new(Sphere::new(
            Point3::random(0., 165.),
            10.,
            white.clone(),
        )));
    }

    world.add(Arc::new(Translate::new(
        Arc::new(RotateY::new(
            Arc::new(BvhNode::new_list(&box2, 0., 1.)),
            15.,
        )),
        Vec3::new(-100., 270., 395.),
    )));

    world
}
