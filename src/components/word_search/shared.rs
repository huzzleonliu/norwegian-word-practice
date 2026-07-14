//! AI 查询共享工具：文本规范化、词性归一、结果合并与批量 CSV 组装。

use crate::components::lexicon_browser::serialize_word_bank_csv;
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

use super::types::GeminiWordResult;

pub(crate) fn normalize_text_opt(value: Option<String>) -> Option<String> {
    let value = value.unwrap_or_default();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn join_pipe(values: Vec<String>) -> Option<String> {
    let list = values
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if list.is_empty() {
        None
    } else {
        Some(list.join(" | "))
    }
}

pub(crate) fn normalize_text_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub(crate) fn hint_to_part_of_speech(hint: &str) -> Option<PartOfSpeech> {
    let prefix = hint.split(['-', '_']).next().unwrap_or_default().trim();
    PartOfSpeech::from_key(prefix).ok()
}

pub(crate) fn normalize_part_of_speech(raw: Option<String>, hint: &str) -> String {
    normalize_part_of_speech_enum(raw, hint)
        .as_key()
        .to_string()
}

pub(crate) fn normalize_part_of_speech_enum(raw: Option<String>, hint: &str) -> PartOfSpeech {
    if let Some(pos) = normalize_text_opt(raw) {
        if let Ok(parsed) = PartOfSpeech::from_key(&pos) {
            return parsed;
        }
    }
    hint_to_part_of_speech(hint).unwrap_or(PartOfSpeech::Noun)
}

pub(crate) fn build_bulk_csv_from_results(
    results: &[GeminiWordResult],
    hint: &str,
) -> Result<String, String> {
    // 将多条 AI 结果转为 CSV 文本，供“多条新增”直接复用。
    let entries = results
        .iter()
        .map(|item| WordBankEntry::try_from((item, hint)))
        .collect::<Result<Vec<_>, _>>()?;
    serialize_word_bank_csv(&entries)
}

pub(crate) fn merge_or_insert_word_result(
    results: &mut Vec<GeminiWordResult>,
    incoming: GeminiWordResult,
) {
    // 以（词性 + base_form）为键合并多来源结果，避免重复行。
    let incoming_pos = normalize_text_opt(incoming.part_of_speech.clone()).unwrap_or_default();
    let incoming_base = normalize_text_opt(incoming.base_form.clone()).unwrap_or_default();
    if incoming_base.is_empty() {
        return;
    }

    if let Some(existing) = results.iter_mut().find(|candidate| {
        normalize_text_opt(candidate.part_of_speech.clone()).unwrap_or_default() == incoming_pos
            && normalize_text_opt(candidate.base_form.clone()).unwrap_or_default() == incoming_base
    }) {
        merge_word_result(existing, incoming);
    } else {
        results.push(incoming);
    }
}

fn merge_word_result(target: &mut GeminiWordResult, source: GeminiWordResult) {
    for value in source.english {
        push_unique_text(&mut target.english, value);
    }
    for value in source.chinese {
        push_unique_text(&mut target.chinese, value);
    }

    if target.part_of_speech.is_none() {
        target.part_of_speech = source.part_of_speech;
    }
    if target.base_form.is_none() {
        target.base_form = source.base_form;
    }

    if target.verb_present_tense.is_none() {
        target.verb_present_tense = source.verb_present_tense;
    }
    if target.verb_past_tense.is_none() {
        target.verb_past_tense = source.verb_past_tense;
    }
    if target.verb_imperative.is_none() {
        target.verb_imperative = source.verb_imperative;
    }
    if target.verb_present_participle.is_none() {
        target.verb_present_participle = source.verb_present_participle;
    }
    if target.verb_past_participle.is_none() {
        target.verb_past_participle = source.verb_past_participle;
    }
    if target.verb_passive_infinitive.is_none() {
        target.verb_passive_infinitive = source.verb_passive_infinitive;
    }
    if target.verb_passive_present.is_none() {
        target.verb_passive_present = source.verb_passive_present;
    }
    if target.verb_passive_past.is_none() {
        target.verb_passive_past = source.verb_passive_past;
    }
    if target.noun_plural.is_none() {
        target.noun_plural = source.noun_plural;
    }
    if target.noun_singular_definite.is_none() {
        target.noun_singular_definite = source.noun_singular_definite;
    }
    if target.noun_plural_definite.is_none() {
        target.noun_plural_definite = source.noun_plural_definite;
    }
    if target.noun_singular_definite_genitive.is_none() {
        target.noun_singular_definite_genitive = source.noun_singular_definite_genitive;
    }
    if target.noun_plural_definite_genitive.is_none() {
        target.noun_plural_definite_genitive = source.noun_plural_definite_genitive;
    }
    if target.noun_singular_indefinite_genitive.is_none() {
        target.noun_singular_indefinite_genitive = source.noun_singular_indefinite_genitive;
    }
    if target.noun_plural_indefinite_genitive.is_none() {
        target.noun_plural_indefinite_genitive = source.noun_plural_indefinite_genitive;
    }
    if target.adjective_feminine_form.is_none() {
        target.adjective_feminine_form = source.adjective_feminine_form;
    }
    if target.adjective_neuter_form.is_none() {
        target.adjective_neuter_form = source.adjective_neuter_form;
    }
    if target.adjective_plural_form.is_none() {
        target.adjective_plural_form = source.adjective_plural_form;
    }
    if target.adjective_comparative.is_none() {
        target.adjective_comparative = source.adjective_comparative;
    }
    if target.adjective_superlative_indefinite.is_none() {
        target.adjective_superlative_indefinite = source.adjective_superlative_indefinite;
    }
    if target.adjective_superlative_definite.is_none() {
        target.adjective_superlative_definite = source.adjective_superlative_definite;
    }
    if target.pronoun_object.is_none() {
        target.pronoun_object = source.pronoun_object;
    }
    if target.pronoun_reflexive.is_none() {
        target.pronoun_reflexive = source.pronoun_reflexive;
    }
    if target.pronoun_plural_subject.is_none() {
        target.pronoun_plural_subject = source.pronoun_plural_subject;
    }
    if target.pronoun_plural_object.is_none() {
        target.pronoun_plural_object = source.pronoun_plural_object;
    }
    if target.pronoun_plural_reflexive.is_none() {
        target.pronoun_plural_reflexive = source.pronoun_plural_reflexive;
    }
    if target.determinative_feminine_form.is_none() {
        target.determinative_feminine_form = source.determinative_feminine_form;
    }
    if target.determinative_neuter_form.is_none() {
        target.determinative_neuter_form = source.determinative_neuter_form;
    }
    if target.determinative_plural_form.is_none() {
        target.determinative_plural_form = source.determinative_plural_form;
    }
    if target.adverb_comparative.is_none() {
        target.adverb_comparative = source.adverb_comparative;
    }
    if target.adverb_superlative.is_none() {
        target.adverb_superlative = source.adverb_superlative;
    }
}

fn push_unique_text(target: &mut Vec<String>, value: String) {
    let normalized = value.trim();
    if normalized.is_empty() {
        return;
    }
    if !target
        .iter()
        .any(|existing| existing.trim().eq_ignore_ascii_case(normalized))
    {
        target.push(normalized.to_string());
    }
}
