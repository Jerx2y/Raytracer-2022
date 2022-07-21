use rand::Rng;

use crate::basic::vec::{Point3, Vec3};

const POINT_COUNT: usize = 256;

#[derive(Clone, Copy)]
pub struct Perlin {
    randvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        let mut randvec = [Vec3::new(0., 0., 0.); POINT_COUNT];
        for i in 0..POINT_COUNT {
            randvec[i] = Vec3::random(-1., 1.).to_unit();
        }
        let perm_x = Perlin::perlin_generate_perm();
        let perm_y = Perlin::perlin_generate_perm();
        let perm_z = Perlin::perlin_generate_perm();
        Self {
            randvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }
    #[allow(clippy::needless_range_loop)]
    fn perlin_generate_perm() -> [usize; POINT_COUNT] {
        let mut p: [usize; POINT_COUNT] = [0; POINT_COUNT];
        for i in 0..POINT_COUNT {
            p[i] = i;
        }
        Perlin::permute(p)
    }

    fn permute(mut p: [usize; POINT_COUNT]) -> [usize; POINT_COUNT] {
        let mut rng = rand::thread_rng();
        for i in (0..POINT_COUNT).rev() {
            let target = rng.gen_range(0..i + 1);
            p.swap(i, target);
        }
        p
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::needless_range_loop)]
    pub fn noise(&self, p: Point3) -> f64 {
        let mut u = p.x - p.x.floor();
        let mut v = p.y - p.y.floor();
        let mut w = p.z - p.z.floor();

        u = u * u * (3. - 2. * u);
        v = v * v * (3. - 2. * v);
        w = w * w * (3. - 2. * w);

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c: [[[Vec3; 2]; 2]; 2] = [[[Vec3::new(0., 0., 0.); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randvec[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]]
                }
            }
        }

        Perlin::trilinear_interp(c, u, v, w)
    }

    #[allow(clippy::needless_range_loop)]
    fn trilinear_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3. - 2. * u);
        let vv = v * v * (3. - 2. * v);
        let ww = w * w * (3. - 2. * w);

        let mut accum = 0.;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += Vec3::dot(c[i][j][k], weight_v)
                        * (i as f64 * uu + (1 - i) as f64 * (1. - uu))
                        * (j as f64 * vv + (1 - j) as f64 * (1. - vv))
                        * (k as f64 * ww + (1 - k) as f64 * (1. - ww));
                }
            }
        }
        accum
    }
    pub fn turb(&self, p: Point3, depth: i32) -> f64 {
        let mut accum = 0.;
        let mut tmp_p = p;
        let mut weight = 1.;

        for _i in 0..depth {
            accum += weight * self.noise(tmp_p);
            weight *= 0.5;
            tmp_p *= 2.;
        }

        accum.abs()
    }
}
