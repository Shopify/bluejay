use bluejay_parser::ast::definition::DefinitionDocument;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_github_schema(c: &mut Criterion) {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    c.bench_function("parse github schema", |b| {
        b.iter(|| DefinitionDocument::parse(black_box(s.as_str())))
    });
}

criterion_group!(benches, parse_github_schema);
criterion_main!(benches);
