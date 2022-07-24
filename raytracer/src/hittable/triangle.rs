use crate::{
    basic::{
        ray::Ray,
        vec::{Point3, Vec3},
    },
    material::Material,
};

use super::{bvh::aabb::AABB, HitRecord, Hittable};

pub struct Triangle<M>
where
    M: Material,
{
    a: Point3,
    b: Point3,
    c: Point3,
    mp: M,
}

impl<M: Material> Triangle<M> {
    pub fn new(x: Point3, y: Point3, z: Point3, mp: M) -> Self {
        Self {
            a: x,
            b: y,
            c: z,
            mp,
        }
    }
    pub fn get_normal(&self) -> Vec3 {
        Vec3::cross(self.b - self.a, self.c - self.a).to_unit()
    }
    pub fn inside(&self, p: Point3) -> bool {
        Vec3::dot(
            Vec3::cross(self.c - self.a, p - self.a),
            Vec3::cross(self.c - self.a, self.b - self.a),
        ) >= 0.
            && Vec3::dot(
                Vec3::cross(self.a - self.b, p - self.b),
                Vec3::cross(self.a - self.b, self.c - self.b),
            ) >= 0.
            && Vec3::dot(
                Vec3::cross(self.b - self.c, p - self.c),
                Vec3::cross(self.b - self.c, self.a - self.c),
            ) >= 0.
    }
}

impl<M: Material> Hittable for Triangle<M> {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let origin = r.orig;
        let direction = r.dir;
        let n = self.get_normal();
        let t = Vec3::dot(direction, n);
        let t = Vec3::dot(self.a - origin, n) / t;
        if t.is_nan() || t < t_min || t > t_max {
            return None;
        }
        let p = origin + direction * t;
        if !self.inside(p) {
            return None;
        }

        let a1 = self.a.x - self.b.x;
        let b1 = self.a.x - self.c.x;
        let c1 = self.a.x - p.x;
        let a2 = self.a.y - self.b.y;
        let b2 = self.a.y - self.c.y;
        let c2 = self.a.y - p.y;
        let beta = (c1 * b2 - b1 * c2) / (a1 * b2 - b1 * a2);
        let gama = (a1 * c2 - a2 * c1) / (a1 * b2 - b1 * a2);

        let mut rec = HitRecord::new(p, n, t, beta, gama, true, &self.mp);
        rec.set_face_normal(r, n);
        Some(rec)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(
                self.a.x.min(self.b.x.min(self.c.x)),
                self.a.y.min(self.b.y.min(self.c.y)),
                self.a.z.min(self.b.z.min(self.c.z)),
            ),
            Point3::new(
                self.a.x.max(self.b.x.max(self.c.x)),
                self.a.y.max(self.b.y.max(self.c.y)),
                self.a.z.max(self.b.z.max(self.c.z)),
            ),
        ))
    }
}
