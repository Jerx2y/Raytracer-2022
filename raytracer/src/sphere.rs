use std::sync::Arc;

use super::hittable::{HitRecord, Hittable};
use super::material::Material;
use super::ray::Ray;
use super::vec::{Point3, Vec3};

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat_ptr: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            mat_ptr,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.orig - self.center;
        let a = r.dir.length_sqr();
        let half_b = Vec3::dot(oc, r.dir);
        let c = oc.length_sqr() - self.radius * self.radius;

        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        let root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            let root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let mut rec = HitRecord::new(
            r.at(root),
            (r.at(root) - self.center) / self.radius,
            root,
            false,
            self.mat_ptr.clone(),
        );

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}
