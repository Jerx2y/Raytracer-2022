use std::{f64::consts::PI, sync::Arc};

use rand::Rng;

use crate::{
    basic::ray::Ray,
    basic::{
        pdf::{CosPdf, Pdf},
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

pub struct Lambertian {
    albedo: Arc<dyn Texture>,
}

impl Lambertian {
    #[allow(dead_code)]
    pub fn new(a: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(a)),
        }
    }
    pub fn new_arc(albedo: Arc<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord::new(
            None,
            self.albedo.value(rec.u, rec.v, rec.p),
            Some(Arc::new(CosPdf::new(rec.normal))),
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

pub struct DiffuseLight {
    emit: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(c: Color) -> Self {
        Self {
            emit: Arc::new(SolidColor::new(c)),
        }
    }
    #[allow(dead_code)]
    pub fn new_arc(emit: Arc<dyn Texture>) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _r_in: Ray, rec: &HitRecord, u: f64, v: f64, p: Point3) -> Color {
        if rec.front_face {
            self.emit.value(u, v, p)
        } else {
            Color::new(0., 0., 0.)
        }
    }
}

pub struct Isotropic {
    #[allow(dead_code)]
    albedo: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new(c: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(c)),
        }
    }
    pub fn new_arc(a: Arc<dyn Texture>) -> Self {
        Self { albedo: a }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord::new(
            Some(Ray::new(rec.p, random_in_unit_sphere(), r_in.tm)),
            self.albedo.value(rec.u, rec.v, rec.p),
            None,
        ))
    }
}

pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: Color,
    pub pdf_ptr: Option<Arc<dyn Pdf>>,
}

impl ScatterRecord {
    pub fn new(
        specular_ray: Option<Ray>,
        attenuation: Color,
        pdf_ptr: Option<Arc<dyn Pdf>>,
    ) -> Self {
        Self {
            specular_ray,
            attenuation,
            pdf_ptr,
        }
    }
}
