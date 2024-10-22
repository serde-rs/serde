// serde\test_suite> cargo expand --all-features --test expand > .\tests\expand\expanded.rs
mod expand {
    mod enum_adjacently_tagged;
    mod enum_internally_tagged;
    mod enum_untagged;
}
