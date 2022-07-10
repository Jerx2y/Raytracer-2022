use crate::vec::random_in_unit_disk;

use super::ray::Ray;
use super::vec::{Point3, Vec3};

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    _w: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).to_unit();
        let u = Vec3::cross(vup, w).to_unit();
        let v = Vec3::cross(w, u);

        let origin = lookfrom;
        let horizontal = u * viewport_width * focus_dist;
        let vertical = v * viewport_height * focus_dist;
        let lower_left_corner = origin - horizontal / 2. - vertical / 2. - w * focus_dist;
        let lens_radius = aperture / 2.;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            _w: w,
            lens_radius,
        }
        //            auto theta = degrees_to_radians(vfov);
        //            auto h = tan(theta/2);
        //            auto viewport_height = 2.0 * h;
        //            auto viewport_width = aspect_ratio * viewport_height;
        //
        //            auto w = unit_vector(lookfrom - lookat);
        //            auto u = unit_vector(cross(vup, w));
        //            auto v = cross(w, u);
        //
        //            origin = lookfrom;
        //            horizontal = viewport_width * u;
        //            vertical = viewport_height * v;
        //            lower_left_corner = origin - horizontal/2 - vertical/2 - w;

        //        let viewport_height = 2.0;
        //        let viewport_width = aspect_ratio * viewport_height;
        //        let focal_length = 1.0;
        //
        //        let origin = Point3::new(0., 0., 0.);
        //        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        //        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        //        let lower_left_corner =
        //            origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + self.horizontal * s + self.vertical * t - self.origin - offset,
        )
    }
}
