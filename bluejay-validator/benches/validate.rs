use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    executable::ExecutableDocument,
    Parse,
};
use bluejay_validator::executable::{document::BuiltinRulesValidator, Cache};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::LazyLock;

const SCHEMA: &str = include_str!("../tests/test_data/executable/schema.graphql");

// A realistic query with fragments, variables, inline fragments, and nested fields
const QUERY_SIMPLE: &str = r#"
query GetDog($command: DogCommand!, $atOtherHomes: Boolean) {
  dog {
    name
    nickname
    barkVolume
    doesKnowCommand(dogCommand: $command)
    isHouseTrained(atOtherHomes: $atOtherHomes)
    owner {
      name
    }
  }
}
"#;

const QUERY_FRAGMENTS: &str = r#"
query GetPetWithFragments($command: DogCommand!) {
  dog {
    ...dogFields
    owner {
      ...humanFields
    }
  }
  pet {
    ...petFields
  }
}

fragment dogFields on Dog {
  name
  nickname
  barkVolume
  doesKnowCommand(dogCommand: $command)
  owner {
    ...humanFields
  }
}

fragment humanFields on Human {
  name
}

fragment petFields on Pet {
  name
  ... on Dog {
    barkVolume
    ...dogFields
  }
  ... on Cat {
    meowVolume
  }
}
"#;

const QUERY_COMPLEX: &str = r#"
query Complex($command: DogCommand!, $atOtherHomes: Boolean) {
  dog {
    ...f1
    ...f2
    ...f3
  }
  pet {
    ...petFragment
  }
}

fragment f1 on Dog {
  name
  nickname
  barkVolume
  doesKnowCommand(dogCommand: $command)
  isHouseTrained(atOtherHomes: $atOtherHomes)
  owner { ...humanFrag }
}

fragment f2 on Dog {
  name
  nickname
  barkVolume
  doesKnowCommand(dogCommand: $command)
  isHouseTrained(atOtherHomes: $atOtherHomes)
  owner { ...humanFrag }
}

fragment f3 on Dog {
  name
  nickname
  barkVolume
  doesKnowCommand(dogCommand: $command)
  isHouseTrained(atOtherHomes: $atOtherHomes)
  owner { ...humanFrag }
}

fragment humanFrag on Human {
  name
}

fragment petFragment on Pet {
  name
  ... on Dog {
    barkVolume
    nickname
    ...f1
  }
  ... on Cat {
    meowVolume
    nickname
  }
}
"#;

static DEFINITION_DOCUMENT: LazyLock<DefinitionDocument<'static>> = LazyLock::new(|| {
    DefinitionDocument::parse(SCHEMA)
        .result
        .expect("Schema had parse errors")
});
static SCHEMA_DEFINITION: LazyLock<SchemaDefinition<'static>> =
    LazyLock::new(|| SchemaDefinition::try_from(&*DEFINITION_DOCUMENT).expect("Schema had errors"));

struct QueryCase {
    name: &'static str,
    source: &'static str,
}

static CASES: &[QueryCase] = &[
    QueryCase {
        name: "simple",
        source: QUERY_SIMPLE,
    },
    QueryCase {
        name: "fragments",
        source: QUERY_FRAGMENTS,
    },
    QueryCase {
        name: "complex",
        source: QUERY_COMPLEX,
    },
];

fn bench_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate");
    for case in CASES {
        let doc = ExecutableDocument::parse(case.source)
            .result
            .expect("Document had parse errors");
        group.bench_with_input(BenchmarkId::from_parameter(case.name), &doc, |b, doc| {
            b.iter(|| {
                let cache = Cache::new(doc, &*SCHEMA_DEFINITION);
                let errors: Vec<_> =
                    BuiltinRulesValidator::validate(doc, &*SCHEMA_DEFINITION, &cache).collect();
                assert!(errors.is_empty());
            });
        });
    }
    group.finish();
}

fn bench_cache_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_construction");
    for case in CASES {
        let doc = ExecutableDocument::parse(case.source)
            .result
            .expect("Document had parse errors");
        group.bench_with_input(BenchmarkId::from_parameter(case.name), &doc, |b, doc| {
            b.iter(|| {
                let _cache = Cache::new(doc, &*SCHEMA_DEFINITION);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_validate, bench_cache_construction);
criterion_main!(benches);
