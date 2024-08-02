use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{self, Rng};
use smallvec::SmallVec;

use abstraction::{histogram::*, k_means::k_means};

fn generate_histograms(n: usize, m: usize) -> Vec<Histogram> {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| {
            Histogram::from(
                (0..m)
                    .map(|_| rng.gen_range(0.0..1.0))
                    .collect::<SmallVec<[f32; 128]>>(),
            )
        })
        .collect::<Vec<Histogram>>()
}

fn bench_1k(c: &mut Criterion) {
    let mut g = c.benchmark_group("K-Means 1k");

    g.bench_function("K-Means: 1k Histograms (MSE)", |b| {
        b.iter_batched(
            || generate_histograms(1000, 10),
            |input| {
                k_means(
                    black_box(10),
                    black_box(1),
                    black_box(input),
                    black_box(avg),
                    black_box(mse),
                )
            },
            criterion::BatchSize::NumBatches(3),
        )
    });

    g.bench_function("K-Means: 1k Histograms (EMD)", |b| {
        b.iter_batched(
            || generate_histograms(1000, 100),
            |input| {
                k_means(
                    black_box(10),
                    black_box(1),
                    black_box(input),
                    black_box(avg),
                    black_box(emd),
                )
            },
            criterion::BatchSize::NumBatches(3),
        )
    });

    g.finish();
}

fn bench_10k(c: &mut Criterion) {
    let mut g = c.benchmark_group("K-Means 10k");

    g.significance_level(0.1).sample_size(10);

    g.bench_function("K-Means: 10k Histograms (MSE)", |b| {
        b.iter_batched(
            || generate_histograms(10000, 10),
            |input| {
                k_means(
                    black_box(100),
                    black_box(1),
                    black_box(input),
                    black_box(avg),
                    black_box(mse),
                )
            },
            criterion::BatchSize::NumBatches(3),
        )
    });

    g.bench_function("K-Means: 10k Histograms (EMD)", |b| {
        b.iter_batched(
            || generate_histograms(10000, 100),
            |input| {
                k_means(
                    black_box(100),
                    black_box(1),
                    black_box(input),
                    black_box(avg),
                    black_box(emd),
                )
            },
            criterion::BatchSize::NumBatches(3),
        )
    });

    g.finish();
}

fn bench_100k(_c: &mut Criterion) {
    let mse_input = generate_histograms(100000, 10);
    let mse_start = std::time::Instant::now();

    k_means(
        black_box(1000),
        black_box(1),
        black_box(mse_input),
        black_box(avg),
        black_box(mse),
    );

    println!("100k Histograms (MSE): {:?}", mse_start.elapsed());

    let emd_input = generate_histograms(100000, 100).clone();
    let emd_start = std::time::Instant::now();

    k_means(
        black_box(1000),
        black_box(1),
        black_box(emd_input),
        black_box(avg),
        black_box(emd),
    );

    println!("100k Histograms (EMD): {:?}", emd_start.elapsed());
}

criterion_group!(benches, bench_100k);
criterion_main!(benches);
