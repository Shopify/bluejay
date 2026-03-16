# Optimization Ideas

- **HashSet+Vec for FieldSelectionMerging cache**: Replacing BTreeMap with HashSet visited + Vec errors showed -26.9% on fsm_128 benchmark but regression on small queries. Could be valuable for production with large schemas/queries.
- **Pre-sized HashMap for grouped_fields**: In `selection_set_contained_fields`, pre-allocate HashMap based on selection set size to reduce rehashing.
- **Arena allocator for FieldContext**: All FieldContext structs are short-lived within a single validation pass. An arena could eliminate per-field allocation overhead.
- **Avoid HashMap in same_for_common_parents_by_name**: For small field groups (2-3), linear scan would avoid HashMap overhead for concrete_groups.
- **Cache field definitions lookup**: The `fields_definition.get(name)` does a linear scan per field. Could cache the lookup result in the Cache struct.
- **Inline small Vecs**: Use `SmallVec<[T; 4]>` for grouped_fields values since most response names have 1-2 fields.
