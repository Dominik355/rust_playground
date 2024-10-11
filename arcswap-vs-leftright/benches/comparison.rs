use arcswap_vs_leftright::{
    left_right_version, ArcSwapVersion, MutexVersion, RwLockVersion, ValueGetter,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_only_writes<V: ValueGetter + 'static>(
    getter: V,
    num_iterations: usize,
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
    let mut group = c.benchmark_group("Atomic-reader");

    let iteration_counts = vec![10_000, 100_000, 500_000, 1_000_000, 5_000_000];

    for num_iterations in iteration_counts {
        group.bench_with_input(
            BenchmarkId::new("Mutex", num_iterations),
            &num_iterations,
            |b, &num_iterations| {
                b.iter(|| bench_only_writes(MutexVersion::default(), num_iterations, 3));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RwLock", num_iterations),
            &num_iterations,
            |b, &num_iterations| {
                b.iter(|| bench_only_writes(RwLockVersion::default(), num_iterations, 3));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ArcSwap", num_iterations),
            &num_iterations,
            |b, &num_iterations| {
                b.iter(|| bench_only_writes(ArcSwapVersion::default(), num_iterations, 3));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Left-Right", num_iterations),
            &num_iterations,
            |b, &num_iterations| {
                b.iter(|| {
                    let (_w, r) = left_right_version();
                    bench_only_writes(r, num_iterations, 3)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
