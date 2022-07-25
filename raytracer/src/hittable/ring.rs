use crate::{material::Material, basic::{vec::{Point3, Vec3}, ray::Ray}};

use super::{Hittable, bvh::aabb::AABB, HitRecord};

pub struct Ring<M>
where
    M: Material
{
    pub r: f64,
    pub t: f64,
    pub mat: M,
    dis_min: f64,
    dis_max: f64,
}

impl<M: Material> Ring<M> {
    pub fn new(r: f64, t: f64, mat: M) -> Self {
        Self {
            r,
            t,
            mat,
            dis_min: (r - t).powi(2),
            dis_max: (r + t).powi(2),
        }
    }
}

impl<M: Material> Hittable for Ring<M> {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = -r.orig.y / r.dir.y;
        if t.is_nan() || t < t_min || t > t_max {
            return None;
        }

        let p = r.at(t);
        let d = p.x.powi(2) + p.z.powi(2);

        if d < self.dis_min || d > self.dis_max {
            return None;
        }

        let mut rec = HitRecord::new(
            p,
            Vec3::new(0., 1., 0.),
            t,
            0.,
            0.,
            false,
            &self.mat,
        );

        rec.set_face_normal(r, Vec3::new(0., 1., 0.));
        Some(rec)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        let thickness = 0.0001;
        Some(AABB::new(
            Point3::new(self.r - self.t, -thickness, self.r - self.t),
            Point3::new(self.r + self.t, thickness, self.r + self.t),
        ))
    }
}