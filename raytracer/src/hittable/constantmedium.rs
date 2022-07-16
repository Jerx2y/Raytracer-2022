use std::{f64::consts::E, sync::Arc};

use rand::Rng;

use crate::{
    basic::ray::Ray,
    basic::vec::{Color, Vec3},
    hittable::bvh::aabb::AABB,
    hittable::{HitRecord, Hittable},
    material::{Isotropic, Material},
    texture::Texture,
};

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    phase_function: Arc<dyn Material>,
    neg_inv_density: f64,
}

impl ConstantMedium {
    #[allow(dead_code)]
    pub fn new_arc(b: Arc<dyn Hittable>, d: f64, a: Arc<dyn Texture>) -> Self {
        Self {
            boundary: b,
            neg_inv_density: -1. / d,
            phase_function: Arc::new(Isotropic::new_arc(a)),
        }
    }
    pub fn new(b: Arc<dyn Hittable>, d: f64, c: Color) -> Self {
        Self {
            boundary: b,
            neg_inv_density: -1. / d,
            phase_function: Arc::new(Isotropic::new(c)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
        if let Some(mut rec1) = self.boundary.hit(r, f64::NEG_INFINITY, f64::INFINITY) {
            if let Some(mut rec2) = self.boundary.hit(r, rec1.t + 0.0001, f64::INFINITY) {
                rec1.t = rec1.t.max(t_min);
                rec2.t = rec2.t.min(t_max);
                if rec1.t >= rec2.t {
                    return None;
                }
                rec1.t = rec1.t.max(0.);
                let ray_length = r.dir.length();
                let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
                let rnd: f64 = rand::thread_rng().gen();
                let hit_distance: f64 = self.neg_inv_density * rnd.log(E);
                if hit_distance > distance_inside_boundary {
                    return None;
                }

                let rec = HitRecord::new(
                    r.at(rec1.t + hit_distance / ray_length),
                    Vec3::new(1., 0., 0.),
                    rec1.t + hit_distance / ray_length,
                    0.,
                    0.,
                    true,
                    self.phase_function.clone(),
                );

                Some(rec)
            } else {
                None
            }
        } else {
            None
        }
    }
}
