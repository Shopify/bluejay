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
- Test snapshot files — must not be modified
- Benchmark harness files — measurement methodology stays the same

## Notes
- `bluejay-core/` is in scope for small, non-breaking changes (no big refactors, no regressions)

## Constraints
- All tests must pass (`cargo test -p bluejay-validator`)
- No new dependencies (keep the crate lean for native extension use)
- Must maintain identical validation behavior (same errors for same inputs)
- Changes should be safe Rust only

## What's Been Tried

### Wins (kept)
1. **HashSet → Vec for parent_fragments** in FieldSelectionMerging (-5.2%) — cheaper clone, linear scan is faster for typical 0-3 items
2. **Eliminated Path Vec allocation** (-6.8%) — `members()` was never called by any rule, made Path Copy
3. **HashSet → Vec in fragment cycle detection** (-1.5%) — same pattern, small N benefits from Vec
4. **Refactored error collection to &mut Vec** (-2.8%) — avoid creating/appending intermediate Vec allocations
5. **Eliminated HashSet in FragmentSpreadIsPossible** (-1.1%) — iterator-based overlap check with fast paths
6. **Optimized duplicates() utility** (-5.3%) — skip BTreeMap allocation when no duplicates found (common case for arguments)
7. **HashMap → linear scan in RequiredArguments** (-2.5%) — argument lists are small (1-5), avoid HashMap overhead
8. **Fast path in same_for_common_parents_by_name** (-1.4%) — skip HashMap grouping when all fields share same parent type

### Dead Ends (discarded)
- **Split cached_errors BTreeMap into HashMap + BTreeMap** — extra HashMap overhead worse than BTreeMap log(n) for small maps
- **Replace BTreeMap cached_errors with HashSet+Vec** — better for large queries (fsm_128 -26.9%) but regression on small queries; might be worth revisiting for production workloads
