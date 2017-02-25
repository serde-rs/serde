extern crate serde_codegen_internals;

use serde_codegen_internals::attr::RenameRule;
use RenameRule::*;

#[test]
fn test_rename_rule_variant_strs() {
    let variants = vec!["Outcome", "VeryTastyVegetables", "A", "Z42", "bad_snake_case"];
    let variants_renamed_expected = vec![
        (PascalCase, vec![Ok("Outcome"), Ok("VeryTastyVegetables"), Ok("A"), Ok("Z42"), Err(())]),
        (CamelCase, vec![Ok("outcome"), Ok("veryTastyVegetables"), Ok("a"), Ok("z42"), Err(())]),
        (SnakeCase, vec![Ok("outcome"), Ok("very_tasty_vegetables"), Ok("a"), Ok("z42"), Err(())]),
        (ScreamingSnakeCase, vec![Ok("OUTCOME"), Ok("VERY_TASTY_VEGETABLES"), Ok("A"), Ok("Z42"), Err(())]),
        (KebabCase, vec![Ok("outcome"), Ok("very-tasty-vegetables"), Ok("a"), Ok("z42"), Err(())]),
    ];

    for variant in variants.iter() {
        assert_eq!(RenameRule::None.apply_to_variant(variant.to_string()), Ok(variant.to_string()));
    }
    for (rule, expected) in variants_renamed_expected.into_iter().map(|(rule, expected)| (rule, expected.into_iter().map(|expect| expect.map(|str| str.to_string())))) {
        for (variant, expected) in variants.iter().zip(expected) {
            assert_eq!(rule.apply_to_variant(variant.to_string()).map_err(|_| ()), expected);
        }
    }
}
