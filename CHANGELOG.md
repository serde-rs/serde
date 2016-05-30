## 0.7.6

NOTES:

* Syncs `serde_codegen` and `serde_macros` with rustc 1.10.0-nightly (7bddce693 2016-05-27).

FEATURES:

* `#[serde(serialize_with=..., deserialize_with=...)]` now supports tuples. #335
* Serde now can be used in `#[no_std]` environments. #316
