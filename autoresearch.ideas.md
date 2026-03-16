# Optimization Ideas

- **HashSet+Vec for FieldSelectionMerging cache**: Replacing BTreeMap with HashSet visited + Vec errors showed -26.9% on fsm_128 benchmark but regression on small queries. Could be valuable for production with large schemas/queries.
- **Pre-sized HashMap for grouped_fields**: In `selection_set_contained_fields`, pre-allocate HashMap based on selection set size to reduce rehashing.
- **Arena allocator for FieldContext**: All FieldContext structs are short-lived within a single validation pass. An arena could eliminate per-field allocation overhead.
- **Cache field definitions lookup**: The `fields_definition.get(name)` does a linear scan per field. Adding a HashMap index at parse time would make this O(1). Blocked by lifetime constraints on `FieldDefinition::name()` trait signature.
- **Combine FieldSelections + orchestrator**: The orchestrator already does `fields_definition.get()` for each field. Moving the "field doesn't exist" error into the orchestrator would save one full selection set iteration.
- **HashMap index on parser's FieldsDefinition**: Would need to change `FieldDefinition::name()` to return `&'a str` instead of `&str`, or use String keys.
- **Ahash or FxHash for internal HashMaps**: Default SipHash is slower than needed for non-adversarial keys. Would require adding a dependency.
