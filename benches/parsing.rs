use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let test_input_str = include_str!("../single.txt");

    c.bench_function("serde", |b| {
        b.iter(|| binance::serde::parse(black_box(test_input_str)))
    });

    c.bench_function("serde_borrowed", |b| {
        b.iter(|| binance::serde_borrowed::parse(black_box(test_input_str)))
    });

    c.bench_function("serde_lazy", |b| {
        b.iter(|| binance::serde_lazy::parse(black_box(test_input_str)))
    });

    c.bench_function("sonic", |b| {
        b.iter(|| binance::sonic::parse(black_box(test_input_str)))
    });

    c.bench_function("custom", |b| {
        b.iter(|| binance::custom::parse(black_box(test_input_str)))
    });

    c.bench_function("custom_lazy", |b| {
        b.iter(|| {
            let document = binance::custom_lazy::Document::new(black_box(test_input_str));
            document
                .as_array()
                .unwrap()
                .get_index(0)
                .unwrap()
                .as_object()
                .unwrap()
                .get_key("amount")
                .unwrap()
                .as_string()
                .unwrap()
                .get_value_as_f64()
                .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
