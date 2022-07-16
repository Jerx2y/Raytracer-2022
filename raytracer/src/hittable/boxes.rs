use std::sync::Arc;

use crate::{
    basic::ray::Ray,
    basic::vec::Point3,
    hittable::aarect::{XYRect, XZRect, YZRect},
    hittable::bvh::aabb::AABB,
    hittable::{Hittable, HittableList},
    material::Material,
};

pub struct Boxes {
    min: Point3,
    max: Point3,
    sides: HittableList,
}

impl Boxes {
    pub fn new(p0: Point3, p1: Point3, ptr: Arc<dyn Material>) -> Self {
        let mut sides: HittableList = Default::default();
        sides.add(Arc::new(XYRect::new(
            p0.x,
            p1.x,
            p0.y,
            p1.y,
            p1.z,
            ptr.clone(),
        )));
        sides.add(Arc::new(XYRect::new(
            p0.x,
            p1.x,
            p0.y,
            p1.y,
            p0.z,
            ptr.clone(),
        )));

        sides.add(Arc::new(XZRect::new(
            p0.x,
            p1.x,
            p0.z,
            p1.z,
            p1.y,
            ptr.clone(),
        )));
        sides.add(Arc::new(XZRect::new(
            p0.x,
            p1.x,
            p0.z,
            p1.z,
            p0.y,
            ptr.clone(),
        )));

        sides.add(Arc::new(YZRect::new(
            p0.y,
            p1.y,
            p0.z,
            p1.z,
            p1.x,
            ptr.clone(),
        )));
        sides.add(Arc::new(YZRect::new(p0.y, p1.y, p0.z, p1.z, p0.x, ptr)));

        Self {
            min: p0,
            max: p1,
            sides,
        }
    }
}

impl Hittable for Boxes {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB::new(self.min, self.max))
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }
}
