pub mod aarect;
pub mod boxes;
pub mod bvh;
pub mod constantmedium;
pub mod sphere;

use std::sync::Arc;

use rand::Rng;

use super::basic::ray::Ray;
use super::basic::vec::{Point3, Vec3};
use super::hittable::bvh::aabb::AABB;
use super::material::Material;

pub struct HitRecord <'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
    pub mat_ptr: &'a dyn Material,
}

impl<'a> HitRecord <'a> {
    pub fn new(
        p: Point3,
        normal: Vec3,
        t: f64,
        u: f64,
        v: f64,
        front_face: bool,
        mat_ptr: &'a dyn Material,
    ) -> Self {
        Self {
            p,
            normal,
            t,
            u,
            v,
            front_face,
            mat_ptr,
        }
    }

    pub fn set_face_normal(&mut self, r: Ray, outward_normal: Vec3) {
        self.front_face = Vec3::dot(r.dir, outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB>;
    fn pdf_value(&self, _o: Point3, _v: Vec3) -> f64 {
        0.
    }
    fn random(&self, _o: Point3) -> Vec3 {
        Vec3::new(1., 0., 0.)
    }
}

#[derive(Clone)]
pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
}

impl Default for HittableList {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
}

impl HittableList {
    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut temp_rec: Option<HitRecord> = None;
        let mut closest_so_far = t_max;
        for object in &self.objects {
            if let Some(rec) = object.hit(r, t_min, closest_so_far) {
                closest_so_far = rec.t;
                temp_rec = Some(rec);
            }
        }
        temp_rec
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }
        let mut output_box = AABB::new(
            Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        );

        for object in &self.objects {
            if let Some(temp_box) = object.bounding_box(time0, time1) {
                output_box = AABB::surrounding_box(output_box, temp_box);
            } else {
                return None;
            }
        }

        Some(output_box)
    }
    fn pdf_value(&self, o: Point3, v: Vec3) -> f64 {
        let len = self.objects.len();
        let mut sum = 0.;
        for i in 0..len {
            sum += self.objects[i].pdf_value(o, v);
        }
        sum / len as f64
    }
    fn random(&self, o: Point3) -> Vec3 {
        let target = rand::thread_rng().gen_range(0..self.objects.len());
        self.objects[target].random(o)
    }
}

pub struct Translate <H>
where H: Hittable {
    ptr: H,
    offset: Vec3,
}

impl<H: Hittable> Translate<H> {
    pub fn new(p: H, displacement: Vec3) -> Self {
        Self {
            ptr: p,
            offset: displacement,
        }
    }
}

impl<H: Hittable> Hittable for Translate<H> {
    #[allow(clippy::manual_map)]
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        if let Some(output_box) = self.ptr.bounding_box(time0, time1) {
            Some(AABB::new(
                output_box.min + self.offset,
                output_box.max + self.offset,
            ))
        } else {
            None
        }
    }

    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let moved_r = Ray::new(r.orig - self.offset, r.dir, r.tm);
        if let Some(mut rec) = self.ptr.hit(moved_r, t_min, t_max) {
            rec.p += self.offset;
            rec.set_face_normal(moved_r, rec.normal);
            Some(rec)
        } else {
            None
        }
    }
}

pub struct RotateY<H>
where H: Hittable {
    ptr: H,
    sin_theta: f64,
    cos_theta: f64,
    aabbox: Option<AABB>,
}

impl<H: Hittable> RotateY<H> {
    pub fn new(p: H, angle: f64) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        if let Some(output_box) = p.bounding_box(0., 1.) {
            let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f64 * output_box.max.x + (1 - i) as f64 * output_box.min.x;
                        let y = j as f64 * output_box.max.y + (1 - j) as f64 * output_box.min.y;
                        let z = k as f64 * output_box.max.z + (1 - k) as f64 * output_box.min.z;

                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vec3::new(newx, y, newz);

                        for c in 0..3 {
                            min[c] = min[c].min(tester[c]);
                            max[c] = max[c].max(tester[c]);
                        }
                    }
                }
            }
            Self {
                ptr: p,
                sin_theta,
                cos_theta,
                aabbox: Some(AABB::new(min, max)),
            }
        } else {
            Self {
                ptr: p,
                sin_theta,
                cos_theta,
                aabbox: None,
            }
        }
    }
}

impl<H: Hittable> Hittable for RotateY<H> {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        self.aabbox
    }
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut origin = r.orig;
        let mut direction = r.dir;

        origin[0] = self.cos_theta * r.orig[0] - self.sin_theta * r.orig[2];
        origin[2] = self.sin_theta * r.orig[0] + self.cos_theta * r.orig[2];

        direction[0] = self.cos_theta * r.dir[0] - self.sin_theta * r.dir[2];
        direction[2] = self.sin_theta * r.dir[0] + self.cos_theta * r.dir[2];

        let rotated_r = Ray::new(origin, direction, r.tm);

        if let Some(mut rec) = self.ptr.hit(rotated_r, t_min, t_max) {
            let mut p = rec.p;
            let mut normal = rec.normal;

            p[0] = self.cos_theta * rec.p[0] + self.sin_theta * rec.p[2];
            p[2] = -self.sin_theta * rec.p[0] + self.cos_theta * rec.p[2];

            normal[0] = self.cos_theta * rec.normal[0] + self.sin_theta * rec.normal[2];
            normal[2] = -self.sin_theta * rec.normal[0] + self.cos_theta * rec.normal[2];

            rec.p = p;
            rec.set_face_normal(rotated_r, normal);

            Some(rec)
        } else {
            None
        }
    }
}

pub struct FlipFace<H>
where H: Hittable {
    ptr: H,
}

impl<H: Hittable> FlipFace<H> {
    pub fn new(ptr: H) -> Self {
        Self { ptr }
    }
}

impl<H: Hittable> Hittable for FlipFace<H> {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if let Some(mut rec) = self.ptr.hit(r, t_min, t_max) {
            rec.front_face = !rec.front_face;
            Some(rec)
        } else {
            None
        }
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.ptr.bounding_box(time0, time1)
    }
}
