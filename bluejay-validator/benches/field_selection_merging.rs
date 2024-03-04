use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use bluejay_validator::executable::document::rules::FieldSelectionMerging;
use bluejay_validator::executable::{document::Orchestrator, Cache};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use once_cell::sync::Lazy;

const SCHEMA: &str = r#"
type Dog {
  name: String!
  nickname: String
  barkVolume: Int
  owner: Human
}

type Human {
  name: String!
  dogs: [Dog!]
}

type Query {
  dog: Dog
  human: Human
}
"#;

fn build_query_string(repetitions: u64) -> String {
    let mut s = String::new();

    (0..repetitions).for_each(|i| {
        s.push_str(format!("fragment f{i} on Query {{ dog {{ name nickname barkVolume owner {{ name dogs {{ name nickname barkVolume }} }} }} }}\n").as_str());
    });

    s.push_str("query MultipleFragmentsSameNames { ");

    (0..repetitions).for_each(|i| {
        s.push_str(format!("...f{i} ").as_str());
    });

    s.push('}');

    s
}

static DEFINITION_DOCUMENT: Lazy<DefinitionDocument<'static>> =
    Lazy::new(|| DefinitionDocument::parse(SCHEMA).expect("Schema had parse errors"));
static SCHEMA_DEFINITION: Lazy<SchemaDefinition<'static>> =
    Lazy::new(|| SchemaDefinition::try_from(&*DEFINITION_DOCUMENT).expect("Schema had errors"));

const REPETITIONS: &[u64] = &[1, 2, 4, 8, 16, 32, 64, 128];

static QUERY_STRINGS: Lazy<Vec<(u64, String)>> = Lazy::new(|| {
    REPETITIONS
        .iter()
        .map(|&repetitions| (repetitions, build_query_string(repetitions)))
        .collect::<Vec<_>>()
});

static EXECUTABLE_DOCUMENTS: Lazy<Vec<(u64, ExecutableDocument<'static>)>> = Lazy::new(|| {
    (*QUERY_STRINGS)
        .iter()
        .map(|(repetitions, query_string)| {
            (
                *repetitions,
                ExecutableDocument::parse(query_string.as_str())
                    .expect("Document had parse errors"),
            )
        })
        .collect::<Vec<_>>()
});

type FieldSelectionMergingValidator<'a, 'e, 's> = Orchestrator<
    'a,
    ExecutableDocument<'e>,
    SchemaDefinition<'s>,
    FieldSelectionMerging<'a, ExecutableDocument<'e>, SchemaDefinition<'s>>,
>;

fn field_selection_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("field_selection_merging");
    for (repetitions, executable_document) in &*EXECUTABLE_DOCUMENTS {
        group.throughput(Throughput::Elements(*repetitions));
        group.bench_with_input(
            BenchmarkId::from_parameter(repetitions),
            &executable_document,
            |b, executable_document| {
                b.iter(|| {
                    let cache = Cache::new(*executable_document, &*SCHEMA_DEFINITION);
                    let _ = FieldSelectionMergingValidator::validate(
                        *executable_document,
                        &*SCHEMA_DEFINITION,
                        &cache,
                    )
                    .collect::<Vec<_>>();
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, field_selection_merging);
criterion_main!(benches);
