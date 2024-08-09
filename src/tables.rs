use std::{
    io::{Read, Write},
    path::Path,
    rc::Rc,
};

use rayon::prelude::*;

use serde::{Deserialize, Serialize};

use smallvec::smallvec;

use poker_evaluator::Evaluator;
use poker_indexer::Indexer;

use crate::histogram::{agg, emd, mse, Histogram};
use crate::k_means::k_means;

const BUCKETS: usize = 47;

pub fn build_strengths(evaluator: &Evaluator) -> Vec<u16> {
    let indexer = Indexer::new(vec![5, 2]);

    let mut strength = vec![0; indexer.count[1] as usize];
    for i in 0..indexer.count[0] {
        let board = indexer.unindex(i, 0)[0];

        let mut list = Vec::new();
        for a in 0..52 {
            for b in 0..52 {
                let hole = 1 << a | 1 << b;
                if a < b && (hole & board) == 0 {
                    list.push((
                        evaluator.evaluate(board | hole),
                        indexer.index(smallvec![board, hole]),
                        (a, b),
                    ));
                }
            }
        }

        list.sort();

        let mut used = vec![0; 52];

        let mut sum = 0;
        for x in list.chunk_by(|a, b| a.0 == b.0) {
            for &(_, _, (a, b)) in x {
                used[a as usize] += 1;
                used[b as usize] += 1;
                sum += 1;
            }

            for &(_, index, (a, b)) in x {
                strength[index as usize] = sum + 1 - used[a as usize] - used[b as usize];
            }

            for &(_, _, (a, b)) in x {
                used[a as usize] += 1;
                used[b as usize] += 1;
                sum += 1;
            }
        }
    }

    strength
}

pub fn generate_flop_histograms(strength: &Vec<u16>) -> Vec<Vec<u16>> {
    let mapper = Indexer::new(vec![5, 2]);

    let indexer = Indexer::new(vec![2, 3]);

    (0..indexer.count[1])
        .into_iter()
        .map(|index| {
            let val = indexer.unindex(index, 1);

            let cards = val[0];
            let board = val[1];

            let mut result = vec![0; BUCKETS];
            for a in 0..52 {
                for b in 0..52 {
                    let next = 1 << a | 1 << b;
                    if a < b && next & (cards | board) == 0 {
                        let i = mapper.index(smallvec![board | next, cards]) as usize;

                        let b = ((strength[i] as f32 / 2162.0) * BUCKETS as f32) as usize;

                        result[b] += 1;
                    }
                }
            }

            result
        })
        .collect()
}

pub fn generate_turn_histograms(strength: &Vec<u16>) -> Vec<Vec<u8>> {
    let mapper = Indexer::new(vec![5, 2]);

    let indexer = Indexer::new(vec![2, 4]);

    (0..indexer.count[1])
        .into_par_iter()
        .map(|index| {
            let val = indexer.unindex(index, 1);

            let cards = val[0];
            let board = val[1];

            let mut result = vec![0; BUCKETS];
            for c in 0..52 {
                if (1 << c) & (cards | board) == 0 {
                    let i = mapper.index(smallvec![board | 1 << c, cards]) as usize;

                    let b = ((strength[i] as f32 / 2162.0) * BUCKETS as f32) as usize;

                    result[b] += 1;
                }
            }

            result
        })
        .collect()
}

pub fn build_ochs_histograms(strength: &Vec<u16>) -> Vec<Histogram> {
    let indexer = Indexer::new(vec![5, 2]);

    let mapper = Indexer::new(vec![2]);

    let mut histograms = vec![Histogram::new(BUCKETS); mapper.count[0] as usize];
    for i in 0..indexer.count[1] {
        let val = indexer.unindex(i, 1);

        let cards = val[1];
        let board = val[0];

        let mut list: Vec<(u64, u64)> = (0..4)
            .map(|p| {
                (
                    cards >> 13 * p & ((1 << 13) - 1),
                    board >> 13 * p & ((1 << 13) - 1),
                )
            })
            .collect();

        list.sort();

        let mut r = 4;
        let mut x = 1;
        for chunk in list.chunk_by(|a, b| a == b) {
            let c = chunk.len() as u64;

            for k in 0..c {
                x *= r - k;
                x /= k + 1;
            }

            r -= c;
        }

        histograms[mapper.index(smallvec![cards]) as usize].put(
            ((strength[i as usize] as f32 / 2162.0) * BUCKETS as f32) as usize,
            x as f32,
        );
    }

    histograms.into_iter().map(|x| x.norm()).collect()
}

pub fn generate_river_histograms(evaluator: &Evaluator, ochs: &Vec<usize>) -> Vec<Histogram> {
    let size = ochs.iter().max().unwrap() + 1;

    let mapper = Indexer::new(vec![2, 5]);

    let indexer = Indexer::new(vec![5]);

    let mut histograms = vec![vec![0.0; size]; mapper.count[1] as usize];
    for i in 0..indexer.count[0] {
        let board = indexer.unindex(i, 0)[0];

        let mut list = Vec::new();
        for a in 0..52 {
            for b in 0..52 {
                let hole = 1 << a | 1 << b;
                if a < b && (hole & board) == 0 {
                    list.push((
                        evaluator.evaluate(board | hole),
                        mapper.index(smallvec![hole, board]) as usize,
                        mapper.index(smallvec![hole]) as usize,
                        (a, b),
                    ));
                }
            }
        }

        list.sort_unstable();

        let mut count = vec![0; size];
        for &(_, _, hole, _) in &list {
            count[ochs[hole]] += 1;
        }

        let mut used = vec![vec![0; 52]; size];

        let mut sum = vec![0; size];
        for x in list.chunk_by(|a, b| a.0 == b.0) {
            for &(_, _, hole, (a, b)) in x {
                used[ochs[hole]][a as usize] += 1;
                used[ochs[hole]][b as usize] += 1;
                sum[ochs[hole]] += 1;
            }

            for &(_, index, hole, (a, b)) in x {
                for k in 0..size {
                    histograms[index][k] = (sum[k] + (ochs[hole] == k) as u32
                        - used[k][a as usize]
                        - used[k][b as usize]) as f32;
                }
            }

            for &(_, _, hole, (a, b)) in x {
                used[ochs[hole]][a as usize] += 1;
                used[ochs[hole]][b as usize] += 1;
                sum[ochs[hole]] += 1;
            }
        }

        list.dedup_by_key(|(_, index, _, _)| index.clone());

        for (_, index, hole, (a, b)) in list {
            for k in 0..size {
                let num = count[k] + (ochs[hole] == k) as u32 - (used[k][a] + used[k][b]) / 2;
                if num != 0 {
                    histograms[index][k] /= num as f32;
                }
                else {
                    histograms[index][k] = 0.0;
                }
            }
        }
    }

    histograms.into_iter().map(|x| Histogram::from(x)).collect()
}

pub fn cluster_flops(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    println!("Getting Flops");

    let flop: Vec<Histogram> = get(
        &(path.clone() + "flop.bin"),
        Box::new({
            let input = Rc::clone(strength);
            move || generate_flop_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()))
    .collect();

    println!("Clustering Flops");

    k_means(count, 20, &flop, agg, emd)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

pub fn cluster_turns(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    println!("Getting Turns");

    let turn: Vec<Histogram> = get(
        path,
        Box::new({
            let input = Rc::clone(strength);
            move || generate_turn_histograms(&input)
        }),
    )
    .into_iter()
    .map(|x| Histogram::from(x.into_iter().map(|x| x.into()).collect()))
    .collect();

    println!("Clustering Turns");

    k_means(count, 5, &turn, agg, emd)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

pub fn cluster_ochs(count: usize, path: &String, strength: &Rc<Vec<u16>>) -> Vec<usize> {
    println!("Getting OCHS");

    let ochs: Vec<Histogram> = get(
        path,
        Box::new({
            let input = Rc::clone(strength);
            move || build_ochs_histograms(&input)
        }),
    );

    println!("Clustering OCHS");

    k_means(count, 95, &ochs, agg, emd)
}

pub fn cluster_rivers(
    count: usize,
    path: &String,
    evaluator: &Rc<Evaluator>,
    ochs: &Rc<Vec<usize>>,
) -> Vec<u16> {
    println!("Getting Rivers");

    let river: Vec<Histogram> = get(
        path,
        Box::new({
            let evaluator = Rc::clone(&evaluator);
            let ochs = Rc::clone(&ochs);
            move || generate_river_histograms(&evaluator, &ochs)
        }),
    );

    println!("Clustering Rivers");

    k_means(count, 1, &river, agg, mse)
        .into_iter()
        .map(|x| x as u16)
        .collect()
}

pub fn get_strengths(path: String, evaluator: &Rc<Evaluator>) -> Vec<u16> {
    println!("Getting Strengths");

    get(
        &path,
        Box::new({
            let evaluator = Rc::clone(evaluator);
            move || build_strengths(&evaluator)
        }),
    )
}

pub fn get_flop_clusters(file: String, path: String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    get(
        &file,
        Box::new({
            let strength = Rc::clone(strength);
            move || cluster_flops(2197, &path, &strength)
        }),
    )
}

pub fn get_turn_clusters(file: String, path: String, strength: &Rc<Vec<u16>>) -> Vec<u16> {
    get(
        &file,
        Box::new({
            let strength = Rc::clone(strength);
            move || cluster_turns(2197, &path, &strength)
        }),
    )
}

pub fn get_ochs_clusters(file: String, path: String, strength: &Rc<Vec<u16>>) -> Vec<usize> {
    get(
        &file,
        Box::new({
            let strength = Rc::clone(strength);
            move || cluster_ochs(13, &path, &strength)
        }),
    )
}

pub fn get_river_clusters(
    file: String,
    path: String,
    evaluator: Rc<Evaluator>,
    ochs: Rc<Vec<usize>>,
) -> Vec<u16> {
    get(
        &file,
        Box::new(move || cluster_rivers(2197, &path, &evaluator, &ochs)),
    )
}

pub fn load<T: for<'d> Deserialize<'d>>(path: &String) -> T {
    let mut buffer = Vec::new();

    std::fs::File::open(path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    bincode::deserialize(&buffer).unwrap()
}

pub fn save<T: Serialize>(path: &String, data: &T) {
    let buffer = bincode::serialize(data).unwrap();

    std::fs::File::create(path)
        .unwrap()
        .write_all(&buffer)
        .unwrap();
}

pub fn get<T: for<'d> Deserialize<'d> + Serialize>(path: &String, f: Box<dyn Fn() -> T>) -> T {
    if Path::new(path).exists() {
        load(path)
    } else {
        let data = f();

        save(path, &data);

        data
    }
}
