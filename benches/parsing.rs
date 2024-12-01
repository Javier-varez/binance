use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let test_input_str = include_str!("../single.txt");

    c.bench_function("very_lazy", |b| {
        b.iter(|| {
            let document = binance::very_lazy::Document::new(black_box(test_input_str));
            document
                .as_array()
                .unwrap()
                .get_index(0)
                .unwrap()
                .as_object()
                .unwrap()
                .get_key("priceChangePercent")
                .unwrap()
                .as_string()
                .unwrap()
                .get_value_as_f64()
                .unwrap()
        })
    });

    c.bench_function("custom", |b| {
        b.iter(|| binance::custom::parse_custom(black_box(test_input_str)))
    });

    c.bench_function("naive", |b| {
        b.iter(|| binance::naive::parse(black_box(test_input_str)))
    });

    c.bench_function("borrowed", |b| {
        b.iter(|| binance::improved::parse(black_box(test_input_str)))
    });

    c.bench_function("lazy", |b| {
        b.iter(|| binance::lazy::parse(black_box(test_input_str)))
    });

    c.bench_function("sonic-rs", |b| {
        b.iter(|| binance::sonic::parse(black_box(test_input_str)))
    });

    c.bench_function("sonic-rs-lazy", |b| {
        b.iter(|| binance::lazy_sonic::parse(black_box(test_input_str)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
