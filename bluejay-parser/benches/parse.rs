use bluejay_parser::ast::{
    definition::{DefaultContext, DefinitionDocument},
    Parse,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn parse(c: &mut Criterion) {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    c.bench_function("parse github schema definitions", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(s.as_str())))
    });

    let s = std::fs::read_to_string("../data/kitchen_sink.graphql").unwrap();
    c.bench_function("parse kitchen sink executable document", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(s.as_str())))
    });
}

criterion_group!(benches, parse);
criterion_main!(benches);
