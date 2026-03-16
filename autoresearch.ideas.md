# Remaining Optimization Ideas

## Still Promising (zero dependency)
- **Share fragment_references across rules**: `AllVariableUsagesAllowed` and `AllVariableUsesDefined` build identical `HashMap<Indexed<FragmentDef>, BTreeSet<PathRoot>>` maps. Could precompute once in Cache by walking the AST for fragment spreads.
- **Precompute field→field_definition mapping**: Build a `HashMap<(&type_name, &field_name), &FieldDefinition>` during `Cache::new()` so both the orchestrator and `FieldSelectionMerging` do O(1) lookups instead of O(n) linear scans via `FieldsDefinition::get()`. Blocked by lifetime constraints on the `FieldDefinition::name()` trait signature (returns `&str` not `&'a str`).

## Promising (needs dependency)
- **FxHash/ahash for internal HashMaps**: Default SipHash is ~2-5x slower than needed for non-adversarial short string keys like field names. Applies to all HashMaps in FieldSelectionMerging and Cache.
- **bumpalo arena for validation pass**: All validation state is scoped to `Orchestrator::validate()`. Threading `&'bump Bump` through Visitor/Rule would allow bump-allocated Vecs and HashMaps. Major refactor but highest potential remaining ceiling (~15-18% of time is pure malloc/free).

## Tried & Exhausted
- Pre-sized HashMap with_capacity — over-allocates for small sets, regression
- Vec-based GroupedFields / SoA — flat on small queries (-32.5% on fsm_128 only)
- Separate HashSet for cycle detection — extra overhead outweighs savings
- #[inline] hints on bluejay-core trait methods — compiler already inlines
- Cache construction optimization — <1% of total time
- Hoisting fields_definition() outside loop — compiler already optimizes
