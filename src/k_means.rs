use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    thread,
};

use rand::prelude::*;

const THREADS: usize = 1;

fn find_centers<T: Clone>(k: usize, a: &Vec<T>, distance: fn(&T, &T) -> f32) -> Vec<T> {
    let mut rng = thread_rng();

    (1..k).fold(vec![a.choose(&mut rng).unwrap().clone()], |mut acc, _| {
        acc.push(
            a.choose_weighted(&mut rng, |x| {
                let mut best = f32::MAX;
                for c in &acc {
                    best = f32::min(best, distance(x, c));
                }
                best
            })
            .unwrap()
            .clone(),
        );
        acc
    })
}

pub fn k_means<T>(
    k: usize,
    m: usize,
    a: Vec<T>,
    merge: fn(Vec<T>) -> T,
    distance: fn(&T, &T) -> f32,
) -> Vec<usize>
where
    T: Debug + Send + Sync + PartialEq + Clone + 'static,
{
    let n = a.len();

    let mut best = f32::MAX;
    let mut vals = vec![0; n];

    let a = Arc::new(a);
    for _ in 0..m {
        let mut c: Arc<Vec<T>> = Arc::new(find_centers(k, &a, distance));

        let positions: Arc<Vec<Mutex<usize>>> = Arc::new((0..n).map(Mutex::new).collect());

        let mut cnt = 0;
        let mut dis;
        loop {
            cnt += 1;

            let mut handles = Vec::new();
            for t in 0..THREADS {
                let a = a.clone();
                let c = c.clone();

                let positions = positions.clone();
                handles.push(thread::spawn(move || {
                    let mut d = 0.0;

                    for i in (t..n).step_by(THREADS) {
                        let mut v = f32::MAX;
                        let mut p = 0;
                        for j in 0..k {
                            let x = distance(&a[i], &c[j]);
                            if x < v {
                                v = x;
                                p = j;
                            }
                        }
                        *positions[i].lock().unwrap() = p;

                        d += v;
                    }

                    d
                }));
            }

            dis = 0.0;
            for handle in handles {
                dis += handle.join().unwrap();
            }

            let mut store: Vec<Vec<T>> = (0..k).map(|_| Vec::new()).collect();
            for i in 0..n {
                store[*positions[i].lock().unwrap()].push(a[i].clone());
            }

            let next = Arc::new(
                store
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| if x.is_empty() { c[i].clone() } else { merge(x) })
                    .collect::<Vec<T>>(),
            );

            // for i in 0..k {
            //     println!("{}: {:?}", i, c[i]);
            // }

            if next == c {
                break;
            }

            c = next;

            break;
        }

        if dis < best {
            best = dis;
            vals = positions.iter().map(|x| *x.lock().unwrap()).collect();
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
            |v| v.iter().sum::<f32>() / v.len() as f32,
            |&x, &y| (x - y).abs(),
        );

        assert!(actual[0] == actual[1]);
        assert!(actual[1] == actual[2]);

        assert!(actual[3] == actual[4]);
        assert!(actual[4] == actual[5]);

        assert!(actual[6] == actual[7]);
        assert!(actual[7] == actual[8]);
    }

    #[test]
    fn test_k_means_decimals() {
        let a = vec![1.91, 13.79, 21.17, 2.35, 3.22, 11.61, 12.62, 22.67, 23.18];

        let actual = k_means(
            3,
            5,
            a,
            |v| v.iter().sum::<f32>() / v.len() as f32,
            |&x, &y| (x - y).abs(),
        );

        assert!(actual[0] == actual[3]);
        assert!(actual[3] == actual[4]);

        assert!(actual[1] == actual[5]);
        assert!(actual[5] == actual[6]);

        assert!(actual[2] == actual[7]);
        assert!(actual[7] == actual[8]);
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
    }
}
