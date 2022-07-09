use super::vec::reflect;
use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec::{random_in_unit_sphere, refract, Color, Vec3},
};

pub trait Material {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(a: Color) -> Self {
        Self { albedo: a }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }
        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(a: Color, f: f64) -> Self {
        Self {
            albedo: a,
            fuzz: if f < 1. { f } else { 1. },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(r_in.dir.to_unit(), rec.normal);
        let scattered = Ray::new(rec.p, reflected + random_in_unit_sphere() * self.fuzz);
        if Vec3::dot(scattered.dir, rec.normal) > 0. {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    pub ir: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            ir: index_of_refraction,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
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
        let direction = if cannot_refract {
            reflect(unit_direction, rec.normal)
        } else {
            refract(unit_direction, rec.normal, refraction_ratio)
        };

        //            double cos_theta = fmin(dot(-unit_direction, rec.normal), 1.0);
        //            double sin_theta = sqrt(1.0 - cos_theta*cos_theta);
        //
        //            bool cannot_refract = refraction_ratio * sin_theta > 1.0;
        //            vec3 direction;
        //
        //            if (cannot_refract)
        //                direction = reflect(unit_direction, rec.normal);
        //            else
        //                direction = refract(unit_direction, rec.normal, refraction_ratio);
        //
        //            scattered = ray(rec.p, direction);

        Some((Color::new(1., 1., 1.), Ray::new(rec.p, direction)))
    }
}
