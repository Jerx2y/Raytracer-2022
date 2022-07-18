use std::{f64::consts::PI, sync::Arc};

use rand::Rng;

use crate::hittable::Hittable;

use super::{
    onb::Onb,
    vec::{Point3, Vec3},
};

pub fn random_cosine_direction() -> Vec3 {
    let mut rng = rand::thread_rng();
    let r1: f64 = rng.gen();
    let r2: f64 = rng.gen();
    let z = (1. - r2).sqrt();
    let phi = 2. * PI * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    Vec3::new(x, y, z)
}

pub trait Pdf {
    fn value(&self, direction: Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct CosPdf {
    uvw: Onb,
}

impl CosPdf {
    #[allow(dead_code)]
    pub fn new(w: Vec3) -> Self {
        Self {
            uvw: Onb::build_from_w(w),
        }
    }
}

impl Pdf for CosPdf {
    fn generate(&self) -> Vec3 {
        self.uvw.local_vec(random_cosine_direction())
    }
    fn value(&self, direction: Vec3) -> f64 {
        let cos = Vec3::dot(direction.to_unit(), self.uvw.w());
        if cos <= 0. {
            0.
        } else {
            cos / PI
        }
    }
}

pub struct HittablePdf {
    o: Point3,
    ptr: Arc<dyn Hittable>,
}

impl HittablePdf {
    pub fn new(ptr: Arc<dyn Hittable>, o: Point3) -> Self {
        Self { o, ptr }
    }
}

impl Pdf for HittablePdf {
    fn generate(&self) -> Vec3 {
        self.ptr.random(self.o)
    }
    fn value(&self, direction: Vec3) -> f64 {
        self.ptr.pdf_value(self.o, direction)
    }
}
