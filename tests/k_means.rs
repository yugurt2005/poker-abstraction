use rand;
use rand_distr::{Distribution, StandardNormal};

use itertools::Itertools;

use poker_abstraction::histogram::{agg, emd, mse, Histogram};
use poker_abstraction::k_means::k_means;

#[test]
fn test_k_means_random() {
    let n = 100;

    let mut rng = rand::thread_rng();

    let mut a = Vec::new();
    for i in 0..n {
        let mut v = vec![0.0; 10];

        for j in 0..10 {
            v[j] = f32::max(0.0, StandardNormal.sample(&mut rng));
        }
        v[i / (n / 10)] += 20.0;

        a.push(Histogram::from(v));
    }

    let centers = k_means(n / 10, 1, &a, agg, mse);

    for (_, chunk) in &centers.iter().chunk_by(|&&x| x) {
        assert_eq!(chunk.count(), n / 10);
    }
}
