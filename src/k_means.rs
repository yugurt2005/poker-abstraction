use core::f32;

use rand::{distributions::WeightedIndex, prelude::*};
use rayon::prelude::*;

use crate::histogram::Histogram;

fn generate_centers(
    k: usize,
    n: usize,
    a: &Vec<Histogram>,
    distance: fn(&Histogram, &Histogram) -> f32,
) -> Vec<Histogram> {
    let mut rng = thread_rng();

    let mut centers = vec![a.choose(&mut rng).unwrap().clone()];

    let mut weights: Vec<f32> = vec![f32::MAX; n];
    for _ in 0..k {
        weights
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, w)| *w = w.min(distance(&a[i], centers.last().unwrap())));

        centers.push(a[WeightedIndex::new(&weights).unwrap().sample(&mut rng)].clone());
    }

    centers
}

pub fn k_means(
    k: usize,
    m: usize,
    a: Vec<Histogram>,
    combines: fn(Option<Histogram>, &Histogram) -> Option<Histogram>,
    distance: fn(&Histogram, &Histogram) -> f32,
) -> Vec<usize> {
    let n = a.len();

    let mut best = f32::MAX;
    let mut idxs = vec![0; n];

    for _ in 0..m {
        let mut centers: Vec<Histogram> = generate_centers(k, n, &a, distance);

        println!("centers generated");

        let mut cnt = 0;
        let mut pre = f32::MAX;
        loop {
            cnt += 1;

            let (pos, dis): (Vec<usize>, Vec<f32>) = a
                .par_iter()
                .enumerate()
                .map(|(_i, h)| {
                    let mut d = f32::MAX;
                    let mut p = 0;
                    for j in 0..k {
                        let x = distance(h, &centers[j]);
                        if x < d {
                            d = x;
                            p = j;
                        }
                    }
                    (p, d)
                })
                .unzip();

            let dis = dis.into_iter().sum::<f32>();

            if dis == pre {
                if dis < best {
                    best = dis;
                    idxs = pos;
                }
                break;
            }

            pre = dis;

            centers.par_iter_mut().enumerate().for_each(|(p, c)| {
                let mut cluster = None;

                for i in 0..n {
                    if pos[i] == p {
                        cluster = combines(cluster, &a[i]);
                    }
                }

                if let Some(x) = cluster {
                    *c = x.norm();
                }
            });

            println!("distance = {}", pre);
        }

        println!(
            "distance = {} (convergence required {} iterations)",
            pre, cnt
        );
    }

    idxs
}

#[cfg(test)]
mod tests {
    use super::*;

    use smallvec::{smallvec, SmallVec};

    use crate::histogram::*;

    #[test]
    fn test_k_means_histograms_mse() {
        let a = vec![
            smallvec![1, 2, 3],
            smallvec![5, 7, 8],
            smallvec![1, 3, 3],
            smallvec![1, 9, 1],
            smallvec![1, 5, 2],
            smallvec![3, 9, 2],
            smallvec![9, 7, 2],
            smallvec![6, 7, 1],
        ]
        .into_iter()
        .map(|v: SmallVec<[i32; 128]>| {
            Histogram::from(v.into_iter().map(|x: i32| x as f32).collect())
        })
        .collect();

        let actual = k_means(3, 5, a, avg, mse);

        assert!(actual[0] == actual[1]);
        assert!(actual[1] == actual[2]);

        assert!(actual[3] == actual[4]);
        assert!(actual[4] == actual[5]);

        assert!(actual[6] == actual[7]);

        assert!(
            actual
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                == 3
        );

        println!("{:?}", actual);
    }

    #[test]
    fn test_k_means_histograms_emd() {
        let a = vec![
            smallvec![1, 2, 3],
            smallvec![5, 7, 8],
            smallvec![1, 3, 3],
            smallvec![1, 9, 1],
            smallvec![1, 5, 2],
            smallvec![3, 9, 2],
            smallvec![9, 7, 2],
            smallvec![6, 7, 1],
        ]
        .into_iter()
        .map(|v: SmallVec<[i32; 128]>| {
            Histogram::from(v.into_iter().map(|x: i32| x as f32).collect())
        })
        .collect();

        let actual = k_means(3, 5, a, avg, emd);

        assert!(actual[0] == actual[1]);
        assert!(actual[1] == actual[2]);

        assert!(actual[3] == actual[4]);
        assert!(actual[4] == actual[5]);

        assert!(actual[6] == actual[7]);

        assert!(
            actual
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                == 3
        );

        println!("{:?}", actual);
    }
}
