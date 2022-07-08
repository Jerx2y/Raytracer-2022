use super::vec::reflect;
use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec::{Color, Vec3},
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
}

impl Metal {
    pub fn new(a: Color) -> Self {
        Self { albedo: a }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(r_in.dir.to_unit(), rec.normal);
        let scattered = Ray::new(rec.p, reflected);
        if Vec3::dot(scattered.dir, rec.normal) > 0. {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}
