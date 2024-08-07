use core::f32;

use rand::prelude::*;
use rayon::prelude::*;

use crate::histogram::Histogram;

fn generate_centers(
    k: usize,
    points: &Vec<Histogram>,
    distance: fn(&Histogram, &Histogram) -> f32,
    rng: &mut ThreadRng,
) -> Vec<Histogram> {
    let mut weights = vec![f32::MAX; points.len()];

    let mut centers = vec![points.choose(rng).unwrap().clone()];
    for _ in 1..k {
        centers.push(
            points[rand::distributions::WeightedIndex::new(
                weights
                    .par_iter_mut()
                    .enumerate()
                    .map(|(i, x)| {
                        *x = x.min(distance(centers.last().unwrap(), &points[i]));
                        *x
                    })
                    .collect::<Vec<f32>>(),
            )
            .unwrap()
            .sample(rng)]
            .clone(),
        );
    }

    centers
}

fn calculate_center_distances(
    centers: &Vec<Histogram>,
    distance: fn(&Histogram, &Histogram) -> f32,
) -> Vec<Vec<f32>> {
    let k = centers.len();

    let mut center_distances = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in 0..k {
            if i < j {
                let d = distance(&centers[i], &centers[j]);
                center_distances[i][j] = d;
                center_distances[j][i] = d;
            }
        }
    }

    center_distances
}

pub fn k_means(
    k: usize,
    m: usize,
    points: &Vec<Histogram>,
    combines: fn(Option<Histogram>, &Histogram) -> Option<Histogram>,
    distance: fn(&Histogram, &Histogram) -> f32,
) -> Vec<usize> {
    let n = points.len();

    let mut best = f32::MAX;
    let mut idxs = vec![0; n];

    println!("clustering {} points into {} clusters", n, k);

    for _ in 0..m {
        let mut centers: Vec<Histogram> = generate_centers(k, &points, distance, &mut thread_rng());

        println!("centers generated");

        let mut cur = vec![0; n];

        let mut cnt = 0;
        let mut pre = f32::MAX;
        loop {
            cnt += 1;

            let center_distances = calculate_center_distances(&centers, distance);

            let (pos, dis): (Vec<usize>, Vec<f32>) = points
                .par_iter()
                .enumerate()
                .map(|(i, h)| {
                    let mut p = cur[i];
                    let mut d = distance(h, &centers[p]);
                    for j in 0..k {
                        if j != p && center_distances[p][j] < 2.0 * d {
                            let x = distance(h, &centers[j]);
                            if x < d {
                                d = x;
                                p = j;
                            }
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

            centers.par_iter_mut().enumerate().for_each(|(p, c)| {
                let mut cluster = None;

                for i in 0..n {
                    if pos[i] == p {
                        cluster = combines(cluster, &points[i]);
                    }
                }

                if let Some(x) = cluster {
                    *c = x.norm();
                }
            });

            pre = dis;
            cur = pos;

            println!("distance = {}", dis);
        }

        println!(
            "distance = {} (convergence required {} iterations)",
            pre, cnt
        );
    }

    println!("best distance = {}", best);

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

        let actual = k_means(3, 5, &a, avg, mse);

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

        let actual = k_means(3, 5, &a, avg, emd);

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
