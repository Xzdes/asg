//! Benchmark для сериализации/десериализации SYN1.
//!
//! Измеряем скорость сохранения и загрузки ASG.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use asg_lang::asg::{ASG, NodeID};
use asg_lang::node_factories::literal_int;
use asg_lang::syn1_writer::save_syn1;
use asg_lang::syn1::load_syn1;
use std::fs;
use std::path::Path;

fn benchmark_serialization(c: &mut Criterion) {
    c.bench_function("serialization/deserialization", |b| {
        b.iter(|| {
            let mut asg = ASG::new();
            for id in 1..1000 {
                asg.add_node(literal_int(id, id as i64));
            }

            let path = "bench_temp.syn1";
            save_syn1(&asg, path).unwrap();
            let _loaded = load_syn1(path).unwrap();

            fs::remove_file(path).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_serialization);
criterion_main!(benches);
