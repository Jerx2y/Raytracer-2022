use std::{f64::INFINITY};

use rand::Rng;

use crate::{
    basic::ray::Ray,
    basic::vec::{Point3, Vec3},
    hittable::bvh::aabb::AABB,
    hittable::{HitRecord, Hittable},
    material::Material,
};

pub struct XYRect<M> 
where M: Material {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mp: M,
}

impl<M: Material> XYRect<M> {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mp: M) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            mp,
        }
    }
}

impl<M: Material> Hittable for XYRect<M> {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
        let t = (self.k - r.orig.z) / r.dir.z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.orig.x + t * r.dir.x;
        let y = r.orig.y + t * r.dir.y;

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let outward_normal = Vec3::new(0., 0., 1.);
        let mut rec = HitRecord::new(
            r.at(t),
            outward_normal,
            t,
            (x - self.x0) / (self.x1 - self.x0),
            (y - self.y0) / (self.y1 - self.y0),
            true,
            &self.mp,
        );

        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}

pub struct XZRect<M>
where M: Material {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    mp: M,
}

impl<M: Material> XZRect<M> {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, mp: M) -> Self {
        Self {
            x0,
            x1,
            z0,
            z1,
            k,
            mp,
        }
    }
}

impl<M: Material> Hittable for XZRect<M> {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1),
        ))
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
        let t = (self.k - r.orig.y) / r.dir.y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.orig.x + t * r.dir.x;
        let z = r.orig.z + t * r.dir.z;

        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let outward_normal = Vec3::new(0., 1., 0.);
        let mut rec = HitRecord::new(
            r.at(t),
            outward_normal,
            t,
            (x - self.x0) / (self.x1 - self.x0),
            (z - self.z0) / (self.z1 - self.z0),
            true,
            &self.mp,
        );

        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }

    fn pdf_value(&self, o: Point3, v: Vec3) -> f64 {
        if let Some(rec) = self.hit(Ray::new(o, v, 0.), 0.001, INFINITY) {
            let area = (self.x1 - self.x0) * (self.z1 - self.z0);
            let dis_sqr = rec.t * rec.t * v.length_sqr();
            let cos = (Vec3::dot(v, rec.normal) / v.length()).abs();
            dis_sqr / (cos * area)
        } else {
            0.
        }
    }

    fn random(&self, origin: Point3) -> Vec3 {
        let mut rng = rand::thread_rng();
        let random_point = Point3::new(
            rng.gen_range(self.x0..self.x1),
            self.k,
            rng.gen_range(self.z0..self.z1),
        );
        random_point - origin
    }
}

pub struct YZRect<M>
where M: Material {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
    mp: M,
}

impl<M: Material> YZRect<M> {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, mp: M) -> Self {
        Self {
            y0,
            y1,
            z0,
            z1,
            k,
            mp,
        }
    }
}

impl<M: Material> Hittable for YZRect<M> {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.k - 0.0001, self.y0, self.z0),
            Point3::new(self.k + 0.0001, self.y1, self.z1),
        ))
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
        let t = (self.k - r.orig.x) / r.dir.x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = r.orig.y + t * r.dir.y;
        let z = r.orig.z + t * r.dir.z;

        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let outward_normal = Vec3::new(1., 0., 0.);
        let mut rec = HitRecord::new(
            r.at(t),
            outward_normal,
            t,
            (y - self.y0) / (self.y1 - self.y0),
            (z - self.z0) / (self.z1 - self.z0),
            true,
            &self.mp,
        );

        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}
