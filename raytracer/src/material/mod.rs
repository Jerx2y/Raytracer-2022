use std::f64::consts::PI;

use rand::Rng;

use crate::{
    basic::ray::Ray,
    basic::{
        pdf::CosPdf,
        vec::{random_in_unit_sphere, reflect, refract, Color, Point3, Vec3},
    },
    hittable::HitRecord,
    texture::{SolidColor, Texture},
};

pub trait Material: Send + Sync {
    fn scatter(&self, _r_in: Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    fn scattering_pdf(&self, _r_in: Ray, _rec: &HitRecord, _scattered: Ray) -> f64 {
        0.
    }
    fn emitted(&self, _r_in: Ray, _rec: &HitRecord, _u: f64, _v: f64, _p: Point3) -> Color {
        Color::new(0., 0., 0.)
    }
}

#[derive(Clone)]
pub struct Lambertian<T>
where
    T: Texture + Clone,
{
    albedo: T,
}

impl<T: Texture + Clone> Lambertian<T> {
    #[allow(dead_code)]
    pub fn new_arc(albedo: T) -> Self {
        Self { albedo }
    }
}

impl Lambertian<SolidColor> {
    pub fn new(a: Color) -> Self {
        Self {
            albedo: SolidColor::new(a),
        }
    }
}

impl<T: Texture + Clone> Material for Lambertian<T> {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord::new(
            None,
            self.albedo.value(rec.u, rec.v, rec.p),
            Some(CosPdf::new(rec.normal)),
        ))
    }
    fn scattering_pdf(&self, _r_in: Ray, rec: &HitRecord, scattered: Ray) -> f64 {
        let cosine = Vec3::dot(rec.normal, scattered.dir.to_unit());
        if cosine < 0. {
            0.
        } else {
            cosine / PI
        }
    }
}

#[derive(Clone, Copy)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    #[allow(dead_code)]
    pub fn new(a: Color, f: f64) -> Self {
        Self {
            albedo: a,
            fuzz: if f < 1. { f } else { 1. },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let reflected = reflect(r_in.dir.to_unit(), rec.normal);
        Some(ScatterRecord::new(
            Some(Ray::new(
                rec.p,
                reflected + random_in_unit_sphere() * self.fuzz,
                0.,
            )),
            self.albedo,
            None,
        ))
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    pub ir: f64,
}

impl Dielectric {
    #[allow(dead_code)]
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            ir: index_of_refraction,
        }
    }

    fn reflectance(cos: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1. - ref_idx) / (1. + ref_idx);
        r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cos).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let refraction_ratio = if rec.front_face {
            1. / self.ir
        } else {
            self.ir
        };
        let unit_direction = r_in.dir.to_unit();
        // let refracted = refract(unit_direction, rec.normal, refraction_ratio);
        let cos_theta = f64::min(Vec3::dot(-unit_direction, rec.normal), 1.);
        let sin_theta = (1. - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.;

        let mut rng = rand::thread_rng();
        let random_double: f64 = rng.gen_range(0.0..1.0);
        let direction = if cannot_refract
            || Dielectric::reflectance(cos_theta, refraction_ratio) > random_double
        {
            reflect(unit_direction, rec.normal)
        } else {
            refract(unit_direction, rec.normal, refraction_ratio)
        };
        Some(ScatterRecord::new(
            Some(Ray::new(rec.p, direction, r_in.tm)),
            Color::new(1., 1., 1.),
            None,
        ))
    }
}

#[derive(Clone)]
pub struct DiffuseLight<T>
where
    T: Texture + Clone,
{
    emit: T,
}

impl<T: Texture + Clone> DiffuseLight<T> {
    #[allow(dead_code)]
    pub fn new_arc(emit: T) -> Self {
        Self { emit }
    }
}

impl DiffuseLight<SolidColor> {
    pub fn new(c: Color) -> Self {
        Self {
            emit: SolidColor::new(c),
        }
    }
}

impl<T: Texture + Clone> Material for DiffuseLight<T> {
    fn emitted(&self, _r_in: Ray, rec: &HitRecord, u: f64, v: f64, p: Point3) -> Color {
        if rec.front_face {
            self.emit.value(u, v, p)
        } else {
            Color::new(0., 0., 0.)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Isotropic<T>
where
    T: Texture + Clone + Copy,
{
    #[allow(dead_code)]
    albedo: T,
}

impl<T: Texture + Clone + Copy> Isotropic<T> {
    pub fn new_arc(a: T) -> Self {
        Self { albedo: a }
    }
}

impl Isotropic<SolidColor> {
    pub fn new(c: Color) -> Self {
        Self {
            albedo: SolidColor::new(c),
        }
    }
}

impl<T: Texture + Clone + Copy> Material for Isotropic<T> {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord::new(
            Some(Ray::new(rec.p, random_in_unit_sphere(), r_in.tm)),
            self.albedo.value(rec.u, rec.v, rec.p),
            None,
        ))
    }
}

#[derive(Clone, Copy)]
pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: Color,
    pub pdf_ptr: Option<CosPdf>,
}

impl ScatterRecord {
    pub fn new(specular_ray: Option<Ray>, attenuation: Color, pdf_ptr: Option<CosPdf>) -> Self {
        Self {
            specular_ray,
            attenuation,
            pdf_ptr,
        }
    }
}
