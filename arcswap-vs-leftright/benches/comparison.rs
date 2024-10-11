use arcswap_vs_leftright::{
    ArcSwapVersion, LeftRightVersion, MutexVersion, RwLockVersion, ValueManipulator,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_with_few_writes<V: ValueManipulator + 'static>(
    getter: V,
    num_iterations: u64,
    parallelism: usize,
) {
    // have 1% of writes
    let write_modulo = (num_iterations as f64 / 0.01) as u64;
    std::thread::scope(|scope| {
        for _ in 0..parallelism {
            let getter = getter.clone();
            scope.spawn(move || {
                for i in 0..num_iterations {
                    if i % write_modulo == 0 {
                        black_box(getter.set_value(i));
                    } else {
                        black_box(getter.get_value());
                    }
                }
            });
        }
    });
}

fn bench_only_reads<V: ValueManipulator + 'static>(
    getter: V,
    num_iterations: u64,
    parallelism: usize,
) {
    std::thread::scope(|scope| {
        for _ in 0..parallelism {
            let getter = getter.clone();
            scope.spawn(move || {
                for _ in 0..num_iterations {
                    black_box(getter.get_value());
                }
            });
        }
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let parallelisms = vec![1, 3, 10];

    for parallelism in parallelisms {
        let mut group = c.benchmark_group(format!("Atomic-reader-{}", parallelism));

        let iteration_counts: Vec<u64> = vec![10_000, 100_000, 500_000, 1_000_000];

        for num_iterations in iteration_counts {
            group.bench_with_input(
                BenchmarkId::new("Mutex-read", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_only_reads(MutexVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("RwLock-read", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_only_reads(RwLockVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("ArcSwap-read", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_only_reads(ArcSwapVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("Left-Right-read", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_only_reads(LeftRightVersion::default(), num_iterations, 3));
                },
            );

            // read + 1% writes
            group.bench_with_input(
                BenchmarkId::new("Mutex-rw", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_with_few_writes(MutexVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("RwLock-rw", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_with_few_writes(RwLockVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("ArcSwap-rw", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| bench_with_few_writes(ArcSwapVersion::default(), num_iterations, 3));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("Left-Right-rw", num_iterations),
                &num_iterations,
                |b, &num_iterations| {
                    b.iter(|| {
                        bench_with_few_writes(LeftRightVersion::default(), num_iterations, 3)
                    });
                },
            );
        }
        group.finish();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
