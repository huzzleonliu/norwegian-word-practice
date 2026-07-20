//! 提交流程服务测试：覆盖重算去重与冲突检测分支。

use crate::components::lexicon_browser::commit_service::{
    CommitError, CommitRequest, prepare_commit,
};
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::dictionary::compute_word_entry_id;

fn base_entry(id: &str, base_form: &str) -> WordBankEntry {
    WordBankEntry {
        id: id.to_string(),
        selected: true,
        part_of_speech: PartOfSpeech::Verb,
        tags: vec![],
        english: vec!["know".to_string()],
        chinese: vec!["知道".to_string()],
        base_form: base_form.to_string(),
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
fn prepare_commit_deduplicates_recomputed_dirty_rows() {
    let mut a = base_entry("old-a", "vite");
    let mut b = base_entry("old-b", "vite");
    // 翻译字段变化不影响哈希，两个 dirty 行会重算成同一 id。
    a.chinese = vec!["知道".to_string()];
    b.chinese = vec!["明白".to_string()];
    let c = base_entry("stable-c", "laere");

    let result = prepare_commit(CommitRequest {
        entries: vec![a, b, c],
        delete_marks: vec![false, false, false],
        dirty_entry_ids: vec!["old-a".to_string(), "old-b".to_string()],
        is_query_mode: false,
    })
    .expect("prepare_commit should succeed");

    assert_eq!(result.summary.deleted_count, 0);
    assert_eq!(result.summary.recalculated_count, 2);
    assert_eq!(result.summary.removed_duplicate_count, 1);
    assert_eq!(result.committed_entries.len(), 2);
}

#[test]
fn prepare_commit_reports_conflict_with_outside_entries() {
    let dirty = base_entry("dirty-old", "vite");
    let dirty_new_id = compute_word_entry_id(&dirty);
    let outside = base_entry(&dirty_new_id, "outside-form");

    let result = prepare_commit(CommitRequest {
        entries: vec![dirty, outside],
        delete_marks: vec![false, false],
        dirty_entry_ids: vec!["dirty-old".to_string()],
        is_query_mode: false,
    });

    match result {
        Err(CommitError::RecomputedIdConflicts { conflicts }) => {
            assert_eq!(conflicts.len(), 1);
            assert!(conflicts[0].contains("新 id 冲突"));
        }
        other => panic!("expected conflict error, got {other:?}"),
    }
}
