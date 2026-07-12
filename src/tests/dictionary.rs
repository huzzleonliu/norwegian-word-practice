use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::dictionary::{compute_word_entry_id, validate_and_prepare_single_entry};

fn base_entry() -> WordBankEntry {
    WordBankEntry {
        id: String::new(),
        selected: true,
        part_of_speech: PartOfSpeech::Verb,
        tags: vec![],
        english: vec!["know".to_string()],
        chinese: vec!["知道".to_string()],
        base_form: "vite".to_string(),
        verb_present_tense: Some("vet".to_string()),
        verb_past_tense: Some("visste".to_string()),
        verb_imperative: Some("vit".to_string()),
        noun_plural: None,
        noun_singular_definite: None,
        noun_plural_definite: None,
        adjective_neuter_form: None,
        adjective_plural_form: None,
        adjective_comparative: None,
        adjective_superlative_indefinite: None,
        adjective_superlative_definite: None,
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

    assert_ne!(compute_word_entry_id(&a), compute_word_entry_id(&b));

    a.verb_present_tense = Some("veit".to_string());
    assert_ne!(compute_word_entry_id(&a), compute_word_entry_id(&base_entry()));
}

#[test]
fn allow_same_base_form_with_different_pos_or_forms() {
    let first = validate_and_prepare_single_entry(base_entry(), &[]).expect("first entry should pass");

    let mut second = base_entry();
    second.part_of_speech = PartOfSpeech::Noun;
    second.verb_present_tense = None;
    second.verb_past_tense = None;
    second.verb_imperative = None;
    second.noun_plural = Some("viter".to_string());
    second.noun_singular_definite = Some("viten".to_string());
    second.noun_plural_definite = Some("vitene".to_string());

    let second = validate_and_prepare_single_entry(second, &[first.clone()])
        .expect("same base form with different lexeme fields should pass");
    assert_ne!(first.id, second.id);
}

#[test]
fn reject_duplicate_lexeme_combination() {
    let first = validate_and_prepare_single_entry(base_entry(), &[]).expect("first entry should pass");
    let duplicate = validate_and_prepare_single_entry(base_entry(), &[first])
        .expect_err("same lexeme fields should be considered duplicate");
    assert!(duplicate.contains("词条重复"));
}
