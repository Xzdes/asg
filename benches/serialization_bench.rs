//! Benchmark for ASG serialization/deserialization.

use asg_lang::parser::parse_expr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_json_serialization(c: &mut Criterion) {
    // Prepare ASG
    let (asg, _) = parse_expr("(* (+ 2 3) (- 10 4))").unwrap();

    c.bench_function("ASG JSON serialization", |b| {
        b.iter(|| black_box(serde_json::to_string(&asg).unwrap()));
    });
}

fn benchmark_json_deserialization(c: &mut Criterion) {
    // Prepare serialized ASG
    let (asg, _) = parse_expr("(* (+ 2 3) (- 10 4))").unwrap();
    let json = serde_json::to_string(&asg).unwrap();

    c.bench_function("ASG JSON deserialization", |b| {
        b.iter(|| black_box(serde_json::from_str::<asg_lang::asg::ASG>(&json).unwrap()));
    });
}

fn benchmark_roundtrip(c: &mut Criterion) {
    c.bench_function("ASG JSON roundtrip", |b| {
        b.iter(|| {
            let (asg, _) = parse_expr("(if (< x 0) (neg x) x)").unwrap();
            let json = serde_json::to_string(&asg).unwrap();
            black_box(serde_json::from_str::<asg_lang::asg::ASG>(&json).unwrap())
        });
    });
}

criterion_group!(
    benches,
    benchmark_json_serialization,
    benchmark_json_deserialization,
    benchmark_roundtrip
);
criterion_main!(benches);
