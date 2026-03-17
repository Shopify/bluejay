use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    Parse,
};
use bluejay_schema_comparator::compare;
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::LazyLock;

fn data_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../data")
        .join(name)
}

fn load_schema(name: &str) -> SchemaDefinition<'static> {
    let src = std::fs::read_to_string(data_path(name)).unwrap();
    let doc: DefinitionDocument = DefinitionDocument::parse(Box::leak(src.into_boxed_str()))
        .result
        .expect("Schema had parse errors");
    let leaked = Box::leak(Box::new(doc));
    SchemaDefinition::try_from(&*leaked).expect("Schema had errors")
}

static ADMIN_OLD: LazyLock<SchemaDefinition<'static>> =
    LazyLock::new(|| load_schema("admin_schema_2024-07_public.graphql"));

static ADMIN_NEW: LazyLock<SchemaDefinition<'static>> =
    LazyLock::new(|| load_schema("admin_schema_2026-01_public.graphql"));

fn compare_schemas(c: &mut Criterion) {
    c.bench_function("admin schema across versions", |b| {
        b.iter(|| compare(&*ADMIN_OLD, &*ADMIN_NEW));
    });
}

criterion_group!(benches, compare_schemas);
criterion_main!(benches);
