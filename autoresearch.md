# Autoresearch: bluejay-validator performance optimization

## Objective
Optimize the `bluejay-validator` crate for performance, focusing on reducing allocations and improving throughput. This crate is used as a native extension in a Ruby/Rails app via a Ruby gem, so every microsecond saved directly impacts request latency.

## Metrics
- **Primary**: total_ns (ns, lower is better) — sum of validate/{simple,fragments,complex} benchmark times
- **Secondary**: validate_simple_ns, validate_fragments_ns, validate_complex_ns, fsm_128_ns

## How to Run
`./autoresearch.sh` — outputs `METRIC name=number` lines.

## Files in Scope
All files in `bluejay-validator/src/`:

### Core validation machinery
- `executable/document/orchestrator.rs` — Main validation orchestrator, drives the visitor pattern
- `executable/document/visitor.rs` — Visitor trait + tuple impls for composing rules
- `executable/document/path.rs` — Path tracking through selections (allocates Vec per level)
- `executable/cache.rs` — Shared cache for fragment definitions and variable types

### Validation rules (highest impact first)
- `executable/document/rules/field_selection_merging.rs` — Most complex rule; HashMap/HashSet heavy, clones HashSet per fragment spread
- `executable/document/rules/all_variable_usages_allowed.rs` — Tracks variable usages across fragments with BTreeMap/BTreeSet
- `executable/document/rules/fragment_spreads_must_not_form_cycles.rs` — Recursive cycle detection, clones HashSet per recursion
- `executable/document/rules/fragment_spread_is_possible.rs` — Type condition checking
- `executable/document/rules/all_variable_uses_defined.rs` — Variable tracking
- `executable/document/rules/all_variables_used.rs` — Variable usage tracking
- Other rules in `executable/document/rules/` — Generally simpler, less allocation

### Supporting files
- `chain_iters.rs` — Iterator chaining utilities
- `value/input_coercion.rs` — Value coercion logic
- `path.rs` — Path type for error reporting

### Benchmark
- `benches/validate.rs` — Full validation benchmark with simple/fragments/complex queries
- `benches/field_selection_merging.rs` — Focused benchmark for the merging rule

## Off Limits
- `bluejay-core/` trait definitions — public API, do not change signatures
- Test snapshot files — must not be modified
- Benchmark harness files — measurement methodology stays the same

## Constraints
- All tests must pass (`cargo test -p bluejay-validator`)
- No new dependencies (keep the crate lean for native extension use)
- Must maintain identical validation behavior (same errors for same inputs)
- Changes should be safe Rust only

## What's Been Tried
(Nothing yet — this is the initial baseline)
