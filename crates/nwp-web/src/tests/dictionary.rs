//! 字典工具测试：覆盖哈希稳定性、差异性与重复词条校验行为。

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::dictionary::{
    compute_word_entry_id, normalize_added_at, validate_and_prepare_single_entry,
};

fn base_entry() -> WordBankEntry {
    WordBankEntry {
        id: String::new(),
        selected: true,
        part_of_speech: PartOfSpeech::Verb,
        tags: vec![],
        english: vec!["know".to_string()],
        chinese: vec!["知道".to_string()],
        base_form: "vite".to_string(),
        added_at: String::new(),
        verb_present_tense: Some("vet".to_string()),
        verb_past_tense: Some("visste".to_string()),
        verb_imperative: Some("vit".to_string()),
        verb_present_participle: Some("vitende".to_string()),
        verb_past_participle: Some("visst".to_string()),
        verb_passive_infinitive: Some("vites".to_string()),
        verb_passive_present: Some("vites".to_string()),
        verb_passive_past: None,
        noun_plural: None,
        noun_singular_definite: None,
        noun_plural_definite: None,
        noun_singular_definite_genitive: None,
        noun_plural_definite_genitive: None,
        noun_singular_indefinite_genitive: None,
        noun_plural_indefinite_genitive: None,
        adjective_feminine_form: None,
        adjective_neuter_form: None,
        adjective_plural_form: None,
        adjective_comparative: None,
        adjective_superlative_indefinite: None,
        adjective_superlative_definite: None,
        pronoun_object: None,
        pronoun_reflexive: None,
        pronoun_plural_subject: None,
        pronoun_plural_object: None,
        pronoun_plural_reflexive: None,
        determinative_feminine_form: None,
        determinative_neuter_form: None,
        determinative_plural_form: None,
        adverb_comparative: None,
        adverb_superlative: None,
    }
}

#[test]
fn hash_id_ignores_translation_fields() {
    let mut a = base_entry();
    let mut b = base_entry();
    a.chinese = vec!["知道".to_string()];
    b.chinese = vec!["明白".to_string()];
    a.english = vec!["know".to_string()];
    b.english = vec!["understand".to_string()];
    a.tags = vec!["A".to_string()];
    b.tags = vec!["B".to_string()];

    assert_eq!(compute_word_entry_id(&a), compute_word_entry_id(&b));
}

#[test]
fn hash_id_differs_when_lexeme_fields_change() {
    let mut a = base_entry();
    let mut b = base_entry();
    b.part_of_speech = PartOfSpeech::Noun;
    b.verb_present_tense = None;
    b.verb_past_tense = None;
    b.verb_imperative = None;
    b.noun_plural = Some("viter".to_string());
    b.noun_singular_definite = Some("viten".to_string());
    b.noun_plural_definite = Some("vitene".to_string());
    b.noun_singular_definite_genitive = Some("vitens".to_string());
    b.noun_plural_definite_genitive = Some("vitenes".to_string());

    assert_ne!(compute_word_entry_id(&a), compute_word_entry_id(&b));

    a.verb_present_tense = Some("veit".to_string());
    assert_ne!(
        compute_word_entry_id(&a),
        compute_word_entry_id(&base_entry())
    );
}

#[test]
fn allow_same_base_form_with_different_pos_or_forms() {
    let first =
        validate_and_prepare_single_entry(base_entry(), &[]).expect("first entry should pass");

    let mut second = base_entry();
    second.part_of_speech = PartOfSpeech::Noun;
    second.verb_present_tense = None;
    second.verb_past_tense = None;
    second.verb_imperative = None;
    second.noun_plural = Some("viter".to_string());
    second.noun_singular_definite = Some("viten".to_string());
    second.noun_plural_definite = Some("vitene".to_string());
    second.noun_singular_definite_genitive = Some("vitens".to_string());
    second.noun_plural_definite_genitive = Some("vitenes".to_string());

    let second = validate_and_prepare_single_entry(second, &[first.clone()])
        .expect("same base form with different lexeme fields should pass");
    assert_ne!(first.id, second.id);
}

#[test]
fn reject_duplicate_lexeme_combination() {
    let first =
        validate_and_prepare_single_entry(base_entry(), &[]).expect("first entry should pass");
    let duplicate = validate_and_prepare_single_entry(base_entry(), &[first])
        .expect_err("same lexeme fields should be considered duplicate");
    assert!(duplicate.contains("词条重复"));
}

#[test]
fn normalize_added_at_converts_legacy_unix_format() {
    assert_eq!(
        normalize_added_at("1785357336.990Z"),
        "2026-07-29T20:35:36.990Z"
    );
    assert_eq!(
        normalize_added_at("2026-07-29T20:36:10.664Z"),
        "2026-07-29T20:36:10.664Z"
    );
    assert_eq!(
        normalize_added_at("2026.07.29 - 20:35"),
        "2026-07-29T20:35:00.000Z"
    );
    assert_eq!(normalize_added_at(""), "");
}

#[test]
fn format_added_at_for_display_uses_dot_date() {
    use crate::utils::dictionary::format_added_at_for_display;
    assert_eq!(
        format_added_at_for_display("2026-07-29T20:35:36.990Z"),
        "2026.07.29 - 20:35"
    );
    assert_eq!(
        format_added_at_for_display("1785357336.990Z"),
        "2026.07.29 - 20:35"
    );
}
