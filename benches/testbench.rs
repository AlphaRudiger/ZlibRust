use criterion::{criterion_group, criterion_main, Criterion};

use rustpng::{self, bitreader::{self, BitReader}};

fn my_benchmark_function(c: &mut Criterion) {
    let data = std::fs::read("/home/rudiger/Desktop/Problem.xcf").expect("file not found");
    c.bench_function("if", |b| {
        b.iter(|| {
            let mut r = BitReader::from_data(data.clone());
            for i in 0..10_000_000 {
                criterion::black_box(r.read_bit());
            }
        });
    });
    c.bench_function("magic", |b| {
        b.iter(|| {
            let mut r = BitReader::from_data(data.clone());
            for i in 0..10_000_000 {
                criterion::black_box(r.read_bit_better());
            }
        });
    });
    c.bench_function("cursed", |b| {
        b.iter(|| {
            let mut r = BitReader::from_data(data.clone());
            for i in 0..10_000_000 {
                criterion::black_box(r.read_bit_cursed());
            }
        });
    });
    c.bench_function("asm", |b| {
        b.iter(|| {
            let mut r = BitReader::from_data(data.clone());
            for i in 0..10_000_000 {
                criterion::black_box(r.read_bit_asm());
            }
        });
    });
}

criterion_group!(benches, my_benchmark_function);
criterion_main!(benches);
