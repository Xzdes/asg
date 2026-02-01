//! Benchmark for interpreter execution.

use asg_lang::interpreter::Interpreter;
use asg_lang::parser::parse_expr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_simple_arithmetic(c: &mut Criterion) {
    c.bench_function("simple arithmetic (+ 1 2)", |b| {
        b.iter(|| {
            let (asg, root_id) = parse_expr("(+ 1 2)").unwrap();
            let mut interpreter = Interpreter::new();
            black_box(interpreter.execute(&asg, root_id).unwrap())
        });
    });
}

fn benchmark_nested_arithmetic(c: &mut Criterion) {
    c.bench_function("nested arithmetic (* (+ 2 3) (- 10 4))", |b| {
        b.iter(|| {
            let (asg, root_id) = parse_expr("(* (+ 2 3) (- 10 4))").unwrap();
            let mut interpreter = Interpreter::new();
            black_box(interpreter.execute(&asg, root_id).unwrap())
        });
    });
}

fn benchmark_variables(c: &mut Criterion) {
    c.bench_function("variables (let x 42) x", |b| {
        b.iter(|| {
            let (asg, root_id) = parse_expr("(let x 42)").unwrap();
            let mut interpreter = Interpreter::new();
            interpreter.execute(&asg, root_id).unwrap();

            let (asg2, root_id2) = parse_expr("x").unwrap();
            black_box(interpreter.execute(&asg2, root_id2).unwrap())
        });
    });
}

fn benchmark_conditionals(c: &mut Criterion) {
    c.bench_function("conditional (if (< 5 10) 100 0)", |b| {
        b.iter(|| {
            let (asg, root_id) = parse_expr("(if (< 5 10) 100 0)").unwrap();
            let mut interpreter = Interpreter::new();
            black_box(interpreter.execute(&asg, root_id).unwrap())
        });
    });
}

criterion_group!(
    benches,
    benchmark_simple_arithmetic,
    benchmark_nested_arithmetic,
    benchmark_variables,
    benchmark_conditionals
);
criterion_main!(benches);
