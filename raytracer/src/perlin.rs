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

    pub fn noise(&self, p: Point3) -> f64 {
        let i = (((4. * p.x) as isize) & 255) as usize;
        let j = (((4. * p.y) as isize) & 255) as usize;
        let k = (((4. * p.z) as isize) & 255) as usize;
        self.randfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }
}
