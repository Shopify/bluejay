use bluejay_parser::ast::{
    definition::{DefaultContext, DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn parse(c: &mut Criterion) {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    c.bench_function("parse github schema definitions", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(s.as_str())))
    });

    let s = std::fs::read_to_string("../data/admin_schema_2026-01_public.graphql").unwrap();
    c.bench_function("parse admin schema definitions", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(s.as_str())))
    });

    let document = DefinitionDocument::<DefaultContext>::parse(s.as_str())
        .result
        .unwrap();
    c.bench_function("build admin schema definition", |b| {
        b.iter(|| SchemaDefinition::try_from(black_box(&document)).unwrap())
    });

    let s = std::fs::read_to_string("../data/kitchen_sink.graphql").unwrap();
    c.bench_function("parse kitchen sink executable document", |b| {
        b.iter(|| ExecutableDocument::parse(black_box(s.as_str())))
    });

    let s = std::fs::read_to_string("../data/large_executable.graphql").unwrap();
    c.bench_function("parse large executable document", |b| {
        b.iter(|| ExecutableDocument::parse(black_box(s.as_str())))
    });
}

/// The same type bodies, written once as definitions and once as extensions, so that the cost
/// of parsing an extension can be compared with the cost of parsing a definition.
fn parse_type_system_extensions(c: &mut Criterion) {
    let mut definitions = String::new();
    let mut extensions = String::new();
    for i in 0..2_000 {
        let body = format!(
            " implements Node @key(fields: \"id\") {{\n  id: ID!\n  \
             field{i}(first: Int = 10, after: String): [Edge!]! @deprecated(reason: \"x\")\n  \
             other{i}: String\n}}\n"
        );
        definitions.push_str(&format!("type Type{i}{body}"));
        extensions.push_str(&format!("extend type Type{i}{body}"));
    }

    c.bench_function("parse 2000 object type definitions", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(definitions.as_str())))
    });
    c.bench_function("parse 2000 object type extensions", |b| {
        b.iter(|| DefinitionDocument::<DefaultContext>::parse(black_box(extensions.as_str())))
    });
}

criterion_group!(benches, parse, parse_type_system_extensions);
criterion_main!(benches);
