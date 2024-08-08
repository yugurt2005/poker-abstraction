use serde::{Deserialize, Serialize};
use textplots::{Chart, Plot, Shape};

#[derive(Clone, Deserialize, Serialize)]
pub struct Histogram {
    pub n: usize,
    pub s: f32,
    pub x: Vec<f32>,
}

impl Histogram {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            s: 0.0,
            x: vec![0.0; n],
        }
    }

    pub fn from(v: Vec<f32>) -> Self 
    {
        Self {
            n: v.len(),
            s: v.iter().sum(),
            x: v.into_iter().map(f32::into).collect(),
        }
    }

    pub fn put(&mut self, i: usize, x: f32) {
        self.s += x;

        self.x[i] += x;
    }

    pub fn get(&self, i: usize) -> f32 {
        if self.s == 0.0 {
            0.0
        }
        else {
            self.x[i]
        }
    }

    pub fn average(mut self, n: usize) -> Self {
        let n = n as f32;

        self.s /= n;
        for i in 0..self.n {
            self.x[i] /= n;
        }

        self
    }

    pub fn norm(mut self) -> Self {
        if self.s == 0.0 {
            return self;
        }

        for i in 0..self.n {
            self.x[i] /= self.s;
        }
        self.s = 1.0;

        self
    }

    pub fn display(&self) {
        let points = self
            .x
            .iter()
            .enumerate()
            .map(|(i, &x)| (i as f32 + 1.0, x))
            .collect::<Vec<_>>();

        Chart::new(100, 30, 0.0, self.n as f32)
            .lineplot(&Shape::Bars(&points))
            .nice();
    }
}

pub fn emd(a: &Histogram, b: &Histogram) -> f32 {
    if a.s != b.s {
    println!("a: {}, b: {}", a.s, b.s);
    }
    assert!(a.s == b.s);

    let n = std::cmp::min(a.n, b.n);

    let mut d = 0.0;
    let mut s = 0.0;
    for i in 0..n {
        s += a.get(i) - b.get(i);
        d += s.abs();
    }

    d
}

pub fn mse(a: &Histogram, b: &Histogram) -> f32 {
    let n = std::cmp::min(a.n, b.n);

    let mut d = 0.0;
    for i in 0..n {
        let delta = a.get(i) - b.get(i);
        d += delta * delta;
    }

    d
}

pub fn agg(input: Option<Histogram>, other: &Histogram) -> Option<Histogram> {
    match input {
        Some(mut h) => {
            for i in 0..h.n {
                h.put(i, other.get(i));
            }
            Some(h)
        }
        None => Some(other.clone()),
    }
}
