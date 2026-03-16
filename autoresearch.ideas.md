# Optimization Ideas

- **HashSet+Vec for FieldSelectionMerging cache**: Replacing BTreeMap with HashSet visited + Vec errors showed -26.9% on fsm_128 benchmark but regression on small queries. Could be valuable for production with large schemas/queries.
- **Pre-sized HashMap for grouped_fields**: In `selection_set_contained_fields`, pre-allocate HashMap based on selection set size to reduce rehashing.
- **Arena allocator for FieldContext**: All FieldContext structs are short-lived within a single validation pass. An arena could eliminate per-field allocation overhead.
- **Cache field definitions lookup**: The `fields_definition.get(name)` does a linear scan per field. Adding a HashMap index at parse time would make this O(1). Blocked by lifetime constraints on `FieldDefinition::name()` trait signature.
- ~~**Combine FieldSelections + orchestrator**~~: Done via `visit_unknown_field` — saved one full iteration + linear scan per field.
- **Vec-based GroupedFields**: Replacing HashMap in field_selection_merging with Vec-based multimap showed -32.5% on fsm_128 but flat on small queries. The `into_groups()` allocates per-group Vecs (same as HashMap). A true arena approach (bumpalo) would help by making all allocations nearly free.
- **bumpalo arena for validation pass**: All validation state is scoped to `Orchestrator::validate()`. A `&'bump Bump` threaded through Visitor/Rule would allow all Vecs and FieldContexts to use bump allocation. Major refactor but highest potential ceiling.
- **HashMap index on parser's FieldsDefinition**: Would need to change `FieldDefinition::name()` to return `&'a str` instead of `&str`, or use String keys.
- **Ahash or FxHash for internal HashMaps**: Default SipHash is slower than needed for non-adversarial keys. Would require adding a dependency.
