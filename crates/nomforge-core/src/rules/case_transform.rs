use crate::error::Result;
use crate::rules::Case;

/// Apply a case transformation to the filename stem.
pub fn apply_case_transform(stem: &str, case: Case) -> Result<String> {
    Ok(match case {
        Case::Upper => stem.to_uppercase(),
        Case::Lower => stem.to_lowercase(),
        Case::Title => title_case(stem),
    })
}

/// Capitalize the first alphanumeric character of each word and lowercase the
/// rest.
fn title_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                for upper in c.to_uppercase() {
                    result.push(upper);
                }
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        } else {
            result.push(c);
            capitalize_next = true;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{RegexCache, RenameContext, RenameRule};

    fn make_ctx(stem: &str) -> RenameContext {
        RenameContext {
            stem: stem.to_string(),
            extension: "txt".to_string(),
            counter: 0,
        }
    }

    // --- Upper case ---

    #[test]
    fn upper_basic() {
        let ctx = make_ctx("hello");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Upper).unwrap(),
            "HELLO"
        );
    }

    #[test]
    fn upper_already_upper() {
        let ctx = make_ctx("HELLO");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Upper).unwrap(),
            "HELLO"
        );
    }

    #[test]
    fn upper_mixed() {
        let ctx = make_ctx("HeLLo");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Upper).unwrap(),
            "HELLO"
        );
    }

    // --- Lower case ---

    #[test]
    fn lower_basic() {
        let ctx = make_ctx("HELLO");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Lower).unwrap(),
            "hello"
        );
    }

    #[test]
    fn lower_already_lower() {
        let ctx = make_ctx("hello");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Lower).unwrap(),
            "hello"
        );
    }

    #[test]
    fn lower_mixed() {
        let ctx = make_ctx("HeLLo");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Lower).unwrap(),
            "hello"
        );
    }

    // --- Title case ---

    #[test]
    fn title_basic() {
        let ctx = make_ctx("hello world");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Hello World"
        );
    }

    #[test]
    fn title_already_titled() {
        let ctx = make_ctx("Hello World");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Hello World"
        );
    }

    #[test]
    fn title_with_separators() {
        let ctx = make_ctx("hello-world_foo.bar");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Hello-World_Foo.Bar"
        );
    }

    #[test]
    fn title_single_char() {
        let ctx = make_ctx("a");
        assert_eq!(apply_case_transform(&ctx.stem, Case::Title).unwrap(), "A");
    }

    #[test]
    fn title_empty() {
        let ctx = make_ctx("");
        assert_eq!(apply_case_transform(&ctx.stem, Case::Title).unwrap(), "");
    }

    #[test]
    fn title_numbers_preserved() {
        let ctx = make_ctx("file 123 test");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "File 123 Test"
        );
    }

    #[test]
    fn title_uppercase_input() {
        let ctx = make_ctx("HELLO world");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Hello World"
        );
    }

    #[test]
    fn title_all_caps() {
        let ctx = make_ctx("DSC 0001 PHOTO");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Dsc 0001 Photo"
        );
    }

    #[test]
    fn title_mixed_input() {
        let ctx = make_ctx("hELLo WoRLD");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Hello World"
        );
    }

    #[test]
    fn title_unicode() {
        let ctx = make_ctx("ÉLAN café");
        assert_eq!(
            apply_case_transform(&ctx.stem, Case::Title).unwrap(),
            "Élan Café"
        );
    }

    // --- Via enum variant ---

    #[test]
    fn via_enum_upper() {
        let rule = RenameRule::CaseTransform(Case::Upper);
        let ctx = make_ctx("hello");
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "HELLO");
    }

    #[test]
    fn via_enum_lower() {
        let rule = RenameRule::CaseTransform(Case::Lower);
        let ctx = make_ctx("HELLO");
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "hello");
    }

    #[test]
    fn via_enum_title() {
        let rule = RenameRule::CaseTransform(Case::Title);
        let ctx = make_ctx("hello world");
        let cache = RegexCache::new();
        assert_eq!(rule.apply(&ctx, &cache).unwrap(), "Hello World");
    }
}
