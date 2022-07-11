
pub struct AABB {
    min: Point3,
    max: Point3,
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn hit(r: Ray, t_min: f64, t_max: f64) -> bool {
        for i in 0..3 {
            let invD = 1. / r.dir[i];
            let mut t0 = (min[i] - r.orig[i]) * invD;
            let mut t1 = (max[i] - r.orig[i]) * invD;
            if invD < 0. {
                t0.swap(t1);
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surrounding_box(box0: Self, box1: Self) -> Self {
        let small = Point3::new(
            f64::min(box0.min.x, box1.min.x),
            f64::min(box0.min.y, box1.min.y),
            f64::min(box0.min.z, box1.min.z),
        );
        let large = Point3::new(
            f64::max(box0.max.x, box1.max.x),
            f64::max(box0.max.y, box1.max.y),
            f64::max(box0.max.z, box1.max.z),
        );
        AABB::new(small, large)
    }
}

