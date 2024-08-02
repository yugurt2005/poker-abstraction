use rand::prelude::*;
use rayon::prelude::*;

fn find_centers<T: Clone>(k: usize, a: &Vec<T>, distance: fn(&T, &T) -> f32) -> Vec<T> {
    let mut rng = thread_rng();

    let mut centers = vec![a.choose(&mut rng).unwrap().clone()];
    for _ in 1..k {
        centers.push(
            a.choose_weighted(&mut rng, |x| {
                let mut best = f32::MAX;
                for c in &centers {
                    best = f32::min(best, distance(x, c));
                }
                best
            })
            .unwrap()
            .clone(),
        );
    }

    centers
}

pub fn k_means<T>(
    k: usize,
    m: usize,
    a: Vec<T>,
    combines: fn(Vec<&T>) -> T,
    distance: fn(&T, &T) -> f32,
) -> Vec<usize>
where
    T: Send + Sync + Clone + std::iter::Sum + 'static,
{
    let n = a.len();

    let mut best = f32::MAX;
    let mut vals = vec![0; n];

    for _ in 0..m {
        let mut centers: Vec<T> = find_centers(k, &a, distance);

        let mut pre = f32::MAX;
        loop {
            let (pos, dis): (Vec<usize>, Vec<f32>) = a
                .par_iter()
                .map(|h| {
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
                    vals = pos;
                }
                break;
            }

            pre = dis;

            centers.par_iter_mut().enumerate().for_each(|(p, c)| {
                let mut cluster = Vec::new();

                for i in 0..n {
                    if pos[i] == p {
                        cluster.push(&a[i]);
                    }
                }

                if cluster.len() > 0 {
                    *c = combines(cluster)
                }
            });
        }

        // println!(
        //     "convergence required {} iterations: produced distance of {}",
        //     cnt, dis
        // );
    }

    // println!("BEST: {}", best);

    vals
}

#[cfg(test)]
mod tests {
    use super::*;

    use smallvec::{smallvec, SmallVec};

    use crate::histogram::*;

    #[test]
    fn test_k_means() {
        let a: Vec<f32> = vec![1, 2, 3, 11, 12, 13, 21, 22, 23]
            .iter()
            .map(|&x| x as f32)
            .collect();

        let actual = k_means(
            3,
            5,
            a,
            |v| v.iter().map(|&&x| x).sum::<f32>() / v.len() as f32,
            |&x, &y| (x - y).abs(),
        );

        assert!(actual[0] == actual[1]);
        assert!(actual[1] == actual[2]);

        assert!(actual[3] == actual[4]);
        assert!(actual[4] == actual[5]);

        assert!(actual[6] == actual[7]);
        assert!(actual[7] == actual[8]);

        println!("{:?}", actual);
    }

    #[test]
    fn test_k_means_decimals() {
        let a = vec![1.91, 13.79, 21.17, 2.35, 3.22, 11.61, 12.62, 22.67, 23.18];

        let actual = k_means(
            3,
            5,
            a,
            |v| v.iter().map(|&&x| x).sum::<f32>() / v.len() as f32,
            |&x, &y| (x - y).abs(),
        );

        assert!(actual[0] == actual[3]);
        assert!(actual[3] == actual[4]);

        assert!(actual[1] == actual[5]);
        assert!(actual[5] == actual[6]);

        assert!(actual[2] == actual[7]);
        assert!(actual[7] == actual[8]);
        
        println!("{:?}", actual);
    }

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
