use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dynamic_vs_generic::{
    DynamicDeserializatorWrapper, GenericDeserializationWrapper, IntDeserializator,
    StringDeserializator,
};

fn bench_dynamic(c: &mut Criterion) {
    let string_wrapper = DynamicDeserializatorWrapper::new(Box::new(StringDeserializator));
    c.bench_function("deserialize string dynamic", |b| {
        b.iter(|| string_wrapper.deserialize(black_box("Hello, world!")))
    });

    let int_wrapper = DynamicDeserializatorWrapper::new(Box::new(IntDeserializator));
    c.bench_function("deserialize int dynamic", |b| {
        b.iter(|| int_wrapper.deserialize(black_box("42")))
    });
}

fn bench_generic(c: &mut Criterion) {
    let string_deserializator = GenericDeserializationWrapper::new(StringDeserializator);
    c.bench_function("deserialize string generic", |b| {
        b.iter(|| string_deserializator.deserialize(black_box("Hello, world!")))
    });

    let int_deserializator = GenericDeserializationWrapper::new(IntDeserializator);
    c.bench_function("deserialize int generic", |b| {
        b.iter(|| int_deserializator.deserialize(black_box("42")))
    });
}

criterion_group!(benches, bench_dynamic, bench_generic);
criterion_main!(benches);
