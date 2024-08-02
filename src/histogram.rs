use smallvec::{smallvec, SmallVec};

pub struct Histogram {
    n: usize,
    s: f32,
    x: SmallVec<[f32; 128]>,
}

impl Histogram {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            s: 0.0,
            x: smallvec![0.0; n],
        }
    }

    pub fn from(v: SmallVec<[f32; 128]>) -> Self {
        let mut h = Self {
            n: v.len(),
            s: v.iter().sum(),
            x: v,
        };

        h.norm();
        h
    }

    pub fn put(&mut self, i: usize, x: f32) {
        self.s += x;
        self.x[i] += x;
    }

    pub fn get(&self, i: usize) -> f32 {
        self.x[i] / self.s
    }

    pub fn norm(&mut self) {
        for i in 0..self.n {
            self.x[i] /= self.s;
        }
        self.s = 1.0;
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            n: self.n,
            s: self.s,
            x: self.x.clone(),
        }
    }
}

impl std::iter::Sum for Histogram {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let mut res = iter.next().unwrap();
        for h in iter {
            for i in 0..res.n {
                res.put(i, h.get(i));
            }
        }
        res
    }
}

pub fn emd(a: &Histogram, b: &Histogram) -> f32 {
    let n = std::cmp::min(a.n, b.n);

    let mut d = 0.0;
    let mut s = 0.0;
    for i in 0..n {
        s += a.get(i) - b.get(i);
        d += s.abs();
    }

    d.into()
}

pub fn mse(a: &Histogram, b: &Histogram) -> f32 {
    let n = std::cmp::min(a.n, b.n);

    let mut d = 0.0;
    for i in 0..n {
        let delta = a.get(i) - b.get(i);
        d += delta * delta;
    }

    d.into()
}

pub fn avg(mut input: Vec<&Histogram>) -> Histogram {
    let mut res = input.pop().unwrap().clone();

    while !input.is_empty() {
        let cur = input.pop().unwrap();
        for i in 0..res.n {
            res.put(i, cur.get(i));
        }
    }

    res.norm();
    res
}