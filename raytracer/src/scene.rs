use std::{sync::Arc};

use rand::Rng;

use crate::{
    basic::vec::{Color, Point3, Vec3},
    hittable::{
        aarect::{XYRect, XZRect, YZRect},
        boxes::Boxes,
        bvh::BvhNode,
        constantmedium::ConstantMedium,
        sphere::{MovingSphere, Sphere},
        triangle::Triangle,
        FlipFace, HittableList, RotateY, Translate, Zoom,
    },
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    texture::{CheckerTexture, ImageTexture, NoiseTexture, ObjTexture},
};

#[allow(dead_code)]
pub fn random_scene() -> HittableList {
    let mut world: HittableList = Default::default();

    let checker = CheckerTexture::new(Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Lambertian::new_arc(checker),
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
                        Lambertian::new(albedo),
                    )));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.);
                    let fuzz = rng.gen_range(0.0..0.5);
                    world.add(Arc::new(Sphere::new(center, 0.2, Metal::new(albedo, fuzz))));
                } else {
                    world.add(Arc::new(Sphere::new(center, 0.2, Dielectric::new(1.5))));
                }
            }
        }
    }

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 1., 0.),
        1.,
        Dielectric::new(1.5),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(-4., 1., 0.),
        1.,
        Lambertian::new(Color::new(0.4, 0.2, 0.1)),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(4., 1., 0.),
        1.,
        Metal::new(Color::new(0.7, 0.6, 0.5), 0.),
    )));

    world
}

#[allow(dead_code)]
pub fn two_spheres() -> HittableList {
    let mut world: HittableList = Default::default();

    let checker = CheckerTexture::new(Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., -10., 0.),
        10.,
        Lambertian::new_arc(checker),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 10., 0.),
        10.,
        Lambertian::new_arc(checker),
    )));

    world
}

#[allow(dead_code)]
pub fn two_perlin_spheres() -> HittableList {
    let mut world: HittableList = Default::default();
    let pertext = NoiseTexture::new(4.);
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Lambertian::new_arc(pertext),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 2., 0.),
        2.,
        Lambertian::new_arc(pertext),
    )));

    world
}

#[allow(dead_code)]
pub fn earth() -> HittableList {
    let earth_texture = ImageTexture::new("input/earthmap.jpg");
    let earth_surface = Lambertian::new_arc(earth_texture);

    let mut world: HittableList = Default::default();

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 0., 0.),
        2.,
        earth_surface,
    )));

    world
}

#[allow(dead_code)]
pub fn simple_light() -> HittableList {
    let mut world: HittableList = Default::default();
    let pertext = NoiseTexture::new(4.);
    world.add(Arc::new(Sphere::new(
        Point3::new(0., -1000., 0.),
        1000.,
        Lambertian::new_arc(pertext),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0., 2., 0.),
        2.,
        Lambertian::new_arc(pertext),
    )));

    let difflight = DiffuseLight::new(Color::new(4., 4., 4.));
    world.add(Arc::new(XYRect::new(3., 5., 1., 3., -2., difflight)));

    world
}

#[allow(dead_code)]
pub fn cornell_box() -> (HittableList, HittableList) {
    let mut world: HittableList = Default::default();

    let light_strong = DiffuseLight::new(Color::new(30., 30., 30.));
    let light_weak = DiffuseLight::new(Color::new(10., 10., 10.));

    let light_top = XZRect::new(
        213.0,
        343.0,
        127.0,
        232.0,
        554.0,
        light_strong.clone(),
    );
//    let light_left = YZRect::new(
//        100.0,
//        300.0,
//        100.0,
//        150.0,
//        554.0,
//        light_weak.clone(),
//    );
//    let light_right = YZRect::new(
//        100.0,
//        300.0,
//        100.0,
//        150.0,
//        1.0,
//        light_weak,
//    );
//    let light_front = XYRect::new(
//        0.0,
//        0.0,
//        555.0,
//        555.0,
//        -801.,
//        light_strong,
//    );

    world.add(Arc::new(FlipFace::new(light_top.clone())));
//    world.add(Arc::new(FlipFace::new(light_left)));
//    world.add(Arc::new(light_right));


    let red = Lambertian::new(Color::new(0.65, 0.05, 0.05));
    let white = Lambertian::new(Color::new(0.73, 0.73, 0.73));
    let green = Lambertian::new(Color::new(0.12, 0.45, 0.15));

    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 555., green)));
    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 0., red)));
    world.add(Arc::new(XZRect::new(0., 555., 0., 555., 0., white.clone())));
    world.add(Arc::new(XZRect::new(
        0.,
        555.,
        0.,
        555.,
        555.,
        white.clone(),
    )));
    world.add(Arc::new(XYRect::new(0., 555., 0., 555., 555., white)));

    // world.add(Arc::new(FlipFace::new(XYRect::new(213., 343., 217., 332., -100., light))));

    //    let box1 = Boxes::new(
    //        Point3::new(0., 0., 0.),
    //        Point3::new(165., 330., 165.),
    //        white,
    //    );
    //    let box1 = RotateY::new(box1, 15.);
    //    let box1 = Arc::new(Translate::new(box1, Vec3::new(265., 0., 295.)));
    //    world.add(box1);
    //
    //    let glass = Dielectric::new(1.5);
    //    world.add(Arc::new(Sphere::new(
    //        Point3::new(190., 90., 190.),
    //        90.,
    //        glass,
    //    )));

    // let mut box2: Arc<dyn Hittable> = Arc::new(Boxes::new(
    //     Point3::new(0., 0., 0.),
    //     Point3::new(165., 165., 165.),
    //     white,
    // ));
    // box2 = Arc::new(RotateY::new(box2, -18.));
    // box2 = Arc::new(Translate::new(box2, Vec3::new(130., 0., 65.)));
    // world.add(box2);

    // world.add(Arc::new(Triangle::new(
    //     Point3::new(310., 450., 310.),
    //     Point3::new(110., 450., 310.),
    //     Point3::new(190., 250., 90.),
    //     Lambertian::new(Color::new(0.25, 0.41, 1.)),
    // )));

    // objects
    get_object(&mut world);

    let mut lights = HittableList::default();
    
    lights.add(Arc::new(light_top));
//    lights.add(Arc::new(light_left));
//    lights.add(Arc::new(light_right));

    (world, lights)
}

#[allow(dead_code)]
pub fn cornell_smoke() -> HittableList {
    let mut world: HittableList = Default::default();

    let red = Lambertian::new(Color::new(0.65, 0.05, 0.05));
    let white = Lambertian::new(Color::new(0.73, 0.73, 0.73));
    let green = Lambertian::new(Color::new(0.12, 0.45, 0.15));
    let light = DiffuseLight::new(Color::new(7., 7., 7.));

    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 555., green)));
    world.add(Arc::new(YZRect::new(0., 555., 0., 555., 0., red)));
    world.add(Arc::new(FlipFace::new(XZRect::new(
        113., 443., 127., 432., 554., light,
    ))));
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

    let box1 = Boxes::new(
        Point3::new(0., 0., 0.),
        Point3::new(165., 330., 165.),
        white.clone(),
    );
    let box1 = RotateY::new(box1, 15.);
    let box1 = Translate::new(box1, Vec3::new(265., 0., 295.));
    world.add(Arc::new(ConstantMedium::new(
        box1,
        0.01,
        Color::new(0., 0., 0.),
    )));

    let box2 = Boxes::new(
        Point3::new(0., 0., 0.),
        Point3::new(165., 165., 165.),
        white,
    );
    let box2 = RotateY::new(box2, -18.);
    let box2 = Translate::new(box2, Vec3::new(130., 0., 65.));
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
    let ground = Lambertian::new(Color::new(0.48, 0.83, 0.53));

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

    let light = DiffuseLight::new(Color::new(7., 7., 7.));
    world.add(Arc::new(FlipFace::new(XZRect::new(
        123., 423., 147., 412., 554., light,
    ))));

    let center1 = Point3::new(400., 400., 200.);
    let center2 = center1 + Vec3::new(25., 0., 0.);
    let moving_sphere_material = Lambertian::new(Color::new(0.7, 0.3, 0.1));
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
        Dielectric::new(1.5),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0., 150., 145.),
        50.,
        Metal::new(Color::new(0.8, 0.8, 0.9), 1.),
    )));

    let boundary = Sphere::new(Point3::new(360., 150., 145.), 70., Dielectric::new(1.5));
    world.add(Arc::new(boundary.clone()));
    world.add(Arc::new(ConstantMedium::new(
        boundary,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));

    let boundary = Sphere::new(Point3::new(0., 0., 0.), 5000., Dielectric::new(1.5));
    world.add(Arc::new(ConstantMedium::new(
        boundary,
        0.0001,
        Color::new(1., 1., 1.),
    )));

    let emat = Lambertian::new_arc(ImageTexture::new("source/earthmap.jpg"));
    world.add(Arc::new(Sphere::new(
        Point3::new(400., 200., 400.),
        100.,
        emat,
    )));

    let pertext = NoiseTexture::new(0.1);
    world.add(Arc::new(Sphere::new(
        Point3::new(220., 280., 300.),
        80.,
        Lambertian::new_arc(pertext),
    )));

    let mut box2: HittableList = Default::default();
    let white = Lambertian::new(Color::new(0.73, 0.73, 0.73));
    let ns = 1000;
    for _i in 0..ns {
        box2.add(Arc::new(Sphere::new(
            Point3::random(0., 165.),
            10.,
            white.clone(),
        )));
    }

    world.add(Arc::new(Translate::new(
        RotateY::new(BvhNode::new_list(&box2, 0., 1.), 15.),
        Vec3::new(-100., 270., 395.),
    )));

    world
}

fn get_object(world: &mut HittableList) {

    // let file_jpg = "source/obj/patrick.png";
    let file_path = "source/HutaoObj/";
    let file_name = file_path.to_string() + "Hutao.obj";

    let obj = tobj::load_obj(
        file_name,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
    );

    assert!(obj.is_ok());

    let (models, _materials) = obj.expect("Failed to load OBJ file");

    // Materials might report a separate loading error if the MTL file wasn't found.
    // If you don't need the materials, you can generate a default here and use that
    // instead.
    let materials = _materials.expect("Failed to load MTL file");

    for (_i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        // if mesh.positions.len() % 9 != 0 {
        // println!("{}", mesh.positions.len());
        // std::process::exit(0);
        // }

        // print!("{}, ", mesh.material_id.unwrap());
        // print!("{}, ", materials[mesh.material_id.unwrap()].name);
        // println!("{}", );

        let mut vertices: Vec<Point3> = Vec::default();
        for v in 0..mesh.positions.len() / 3 {
            let x = mesh.positions[3 * v] as f64;
            let y = mesh.positions[3 * v + 1] as f64;
            let z = mesh.positions[3 * v + 2] as f64;
            vertices.push(Point3::new(x, y, z));
        }
        let mut object = HittableList::default();
        let mut file_jpg = file_path.to_string();
        file_jpg += materials[mesh.material_id.unwrap()].diffuse_texture.as_str();
        let image = Arc::new(image::open(file_jpg).expect("failed").to_rgb8());

        for v in 0..mesh.indices.len() / 3 {
            let x = mesh.indices[v * 3] as usize;
            let y = mesh.indices[v * 3 + 1] as usize;
            let z = mesh.indices[v * 3 + 2] as usize;

            let u1 = mesh.texcoords[(x * 2)] as f64;
            let v1 = mesh.texcoords[(x * 2 + 1)] as f64;
            let u2 = mesh.texcoords[(y * 2)] as f64;
            let v2 = mesh.texcoords[(y * 2 + 1)] as f64;
            let u3 = mesh.texcoords[(z * 2)] as f64;
            let v3 = mesh.texcoords[(z * 2 + 1)] as f64;
            
            let tex_ptr = ObjTexture::new(u1, v1, u2, v2, u3, v3, image.clone());

            let tri = Triangle::new(
                vertices[x],
                vertices[y],
                vertices[z],
                Lambertian::new_arc(tex_ptr),
                //Lambertian::new(Color::new(0.75, 0.75, 0.75)),
            );
            object.add(Arc::new(tri));
        }

        let object = BvhNode::new_list(&object, 0., 1.);
        let object = Zoom::new(object, 20.);
        let object = RotateY::new(object, 0.);
        let object = Translate::new(object, Vec3::new(278., 0., 500.));
        world.add(Arc::new(object));
    }

    //    for (i, m) in materials.iter().enumerate() {
    //        println!("material[{}].name = \'{}\'", i, m.name);
    //        println!(
    //            "    material.Ka = ({}, {}, {})",
    //            m.ambient[0], m.ambient[1], m.ambient[2]
    //        );
    //        println!(
    //            "    material.Kd = ({}, {}, {})",
    //            m.diffuse[0], m.diffuse[1], m.diffuse[2]
    //        );
    //        println!(
    //            "    material.Ks = ({}, {}, {})",
    //            m.specular[0], m.specular[1], m.specular[2]
    //        );
    //        println!("    material.Ns = {}", m.shininess);
    //        println!("    material.d = {}", m.dissolve);
    //        println!("    material.map_Ka = {}", m.ambient_texture);
    //        println!("    material.map_Kd = {}", m.diffuse_texture);
    //        println!("    material.map_Ks = {}", m.specular_texture);
    //        println!("    material.map_Ns = {}", m.shininess_texture);
    //        println!("    material.map_Bump = {}", m.normal_texture);
    //        println!("    material.map_d = {}", m.dissolve_texture);
    //
    //        for (k, v) in &m.unknown_param {
    //            println!("    material.{} = {}", k, v);
    //        }
    //    }
}
