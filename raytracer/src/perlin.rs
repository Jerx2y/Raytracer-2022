use rand::Rng;

use crate::vec::Point3;

const POINT_COUNT: usize = 256;

pub struct Perlin {
    randfloat: [f64; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        let mut randfloat = [0.; POINT_COUNT];
        let mut rng = rand::thread_rng();
        for i in 0..POINT_COUNT {
            randfloat[i] = rng.gen();
        }
        let perm_x = Perlin::perlin_generate_perm();
        let perm_y = Perlin::perlin_generate_perm();
        let perm_z = Perlin::perlin_generate_perm();
        Self {
            randfloat,
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

        let mut c: [[[f64; 2]; 2]; 2] = Default::default();

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randfloat[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]]
                }
            }
        }

        Perlin::trilinear_interp(c, u, v, w)
    }

    #[allow(clippy::needless_range_loop)]
    fn trilinear_interp(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    accum += (i as f64 * u + (1 - i) as f64 * (1. - u))
                        * (j as f64 * v + (1 - j) as f64 * (1. - v))
                        * (k as f64 * w + (1 - k) as f64 * (1. - w))
                        * c[i][j][k];
                }
            }
        }
        accum
    }
    //        static double trilinear_interp(double c[2][2][2], double u, double v, double w) {
    //            auto accum = 0.0;
    //            for (int i=0; i < 2; i++)
    //                for (int j=0; j < 2; j++)
    //                    for (int k=0; k < 2; k++)
    //                        accum += (i*u + (1-i)*(1-u))*
    //                                (j*v + (1-j)*(1-v))*
    //                                (k*w + (1-k)*(1-w))*c[i][j][k];
    //
    //            return accum;
    //        }
}
