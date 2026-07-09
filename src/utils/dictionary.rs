use std::collections::HashSet;

use crate::components::lexicon_browser::{PART_OF_SPEECH_OPTIONS, WordEntry};
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

pub type SingleEntryDraft = WordBankEntry;

pub fn parse_part_of_speech(raw: &str) -> Result<PartOfSpeech, String> {
    match raw.trim() {
        "verb" => Ok(PartOfSpeech::Verb),
        "noun" => Ok(PartOfSpeech::Noun),
        "adjective" => Ok(PartOfSpeech::Adjective),
        "adverb" => Ok(PartOfSpeech::Adverb),
        "cardinal_number" => Ok(PartOfSpeech::CardinalNumber),
        "ordinal_number" => Ok(PartOfSpeech::OrdinalNumber),
        "month" => Ok(PartOfSpeech::Month),
        "pronoun" => Ok(PartOfSpeech::Pronoun),
        "interrogative" => Ok(PartOfSpeech::Interrogative),
        _ => Err(format!(
            "词性不合法，请从预设选项中选择：{}",
            PART_OF_SPEECH_OPTIONS.join(", ")
        )),
    }
}

pub fn validate_and_prepare_single_entry(
    draft: SingleEntryDraft,
    existing_entries: &[WordEntry],
) -> Result<WordEntry, String> {
    let base_form = draft.base_form.trim().to_string();
    if base_form.is_empty() {
        return Err("原型不能为空。".to_string());
    }
    if existing_entries.iter().any(|entry| entry.base_form.trim() == base_form) {
        return Err(format!("原型 `{base_form}` 已存在，不能重复插入。"));
    }

    validate_forms_by_part_of_speech(&draft.part_of_speech, &draft)?;

    let chinese = normalize_vec(draft.chinese);
    if chinese.is_empty() {
        return Err("对应中文不能为空。".to_string());
    }

    let existing_ids = existing_entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<HashSet<_>>();
    let id = make_unique_id_from_base_form(&base_form, &existing_ids);

    Ok(WordEntry {
        id,
        selected: draft.selected,
        part_of_speech: part_of_speech_to_key(&draft.part_of_speech).to_string(),
        tags: normalize_vec(draft.tags),
        english: normalize_vec(draft.english),
        chinese,
        base_form,
        past_tense: normalize_optional(draft.past_tense),
        imperative: normalize_optional(draft.imperative),
        plural: normalize_optional(draft.plural),
        singular_definite: normalize_optional(draft.singular_definite),
        plural_definite: normalize_optional(draft.plural_definite),
        neuter_form: normalize_optional(draft.neuter_form),
        plural_form: normalize_optional(draft.plural_form),
        adjective_comparative: normalize_optional(draft.adjective_comparative),
        adjective_superlative_indefinite: normalize_optional(draft.adjective_superlative_indefinite),
        adjective_superlative_definite: normalize_optional(draft.adjective_superlative_definite),
        adverb_comparative: normalize_optional(draft.adverb_comparative),
        adverb_superlative: normalize_optional(draft.adverb_superlative),
    })
}

pub fn validate_existing_entry(
    entry: &WordEntry,
    existing_entries: &[WordEntry],
    self_index: usize,
) -> Result<(), String> {
    let base_form = entry.base_form.trim().to_string();
    if base_form.is_empty() {
        return Err("原型不能为空。".to_string());
    }
    if existing_entries
        .iter()
        .enumerate()
        .any(|(idx, item)| idx != self_index && item.base_form.trim() == base_form)
    {
        return Err(format!("原型 `{base_form}` 已存在，不能重复。"));
    }

    let draft = draft_from_word_entry(entry)?;
    let chinese = normalize_vec(draft.chinese.clone());
    if chinese.is_empty() {
        return Err("对应中文不能为空。".to_string());
    }

    validate_forms_by_part_of_speech(&draft.part_of_speech, &draft)
}

pub fn draft_from_word_entry(entry: &WordEntry) -> Result<SingleEntryDraft, String> {
    Ok(SingleEntryDraft {
        id: entry.id.clone(),
        selected: entry.selected,
        part_of_speech: parse_part_of_speech(&entry.part_of_speech)?,
        tags: entry.tags.clone(),
        english: entry.english.clone(),
        chinese: entry.chinese.clone(),
        base_form: entry.base_form.clone(),
        past_tense: entry.past_tense.clone(),
        imperative: entry.imperative.clone(),
        plural: entry.plural.clone(),
        singular_definite: entry.singular_definite.clone(),
        plural_definite: entry.plural_definite.clone(),
        neuter_form: entry.neuter_form.clone(),
        plural_form: entry.plural_form.clone(),
        adjective_comparative: entry.adjective_comparative.clone(),
        adjective_superlative_indefinite: entry.adjective_superlative_indefinite.clone(),
        adjective_superlative_definite: entry.adjective_superlative_definite.clone(),
        adverb_comparative: entry.adverb_comparative.clone(),
        adverb_superlative: entry.adverb_superlative.clone(),
    })
}

fn validate_forms_by_part_of_speech(pos: &PartOfSpeech, draft: &SingleEntryDraft) -> Result<(), String> {
    match pos {
        PartOfSpeech::Verb => {
            require_option("过去式", &draft.past_tense)?;
            require_option("祈使式", &draft.imperative)?;
        }
        PartOfSpeech::Noun => {
            require_option("复数", &draft.plural)?;
            require_option("单数特指", &draft.singular_definite)?;
            require_option("复数特指", &draft.plural_definite)?;
        }
        PartOfSpeech::Adjective => {
            require_option("对应中性", &draft.neuter_form)?;
            require_option("对应复数", &draft.plural_form)?;
            require_option("形容词比较级", &draft.adjective_comparative)?;
            require_option("形容词最高级泛指", &draft.adjective_superlative_indefinite)?;
            require_option("形容词最高级特指", &draft.adjective_superlative_definite)?;
        }
        PartOfSpeech::Adverb => {
            require_option("副词比较级", &draft.adverb_comparative)?;
            require_option("副词最高级", &draft.adverb_superlative)?;
        }
        PartOfSpeech::CardinalNumber
        | PartOfSpeech::OrdinalNumber
        | PartOfSpeech::Month
        | PartOfSpeech::Pronoun
        | PartOfSpeech::Interrogative => {}
    }
    Ok(())
}

fn part_of_speech_to_key(pos: &PartOfSpeech) -> &'static str {
    match pos {
        PartOfSpeech::Verb => "verb",
        PartOfSpeech::Noun => "noun",
        PartOfSpeech::Adjective => "adjective",
        PartOfSpeech::Adverb => "adverb",
        PartOfSpeech::CardinalNumber => "cardinal_number",
        PartOfSpeech::OrdinalNumber => "ordinal_number",
        PartOfSpeech::Month => "month",
        PartOfSpeech::Pronoun => "pronoun",
        PartOfSpeech::Interrogative => "interrogative",
    }
}

fn require_option(field_name: &str, value: &Option<String>) -> Result<(), String> {
    if normalize_optional(value.clone()).is_none() {
        return Err(format!("{field_name}不能为空。"));
    }
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_vec(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn make_unique_id_from_base_form(base_form: &str, existing_ids: &HashSet<&str>) -> String {
    if !existing_ids.contains(base_form) {
        return base_form.to_string();
    }

    let mut index = 2_usize;
    loop {
        let candidate = format!("{base_form}-{index}");
        if !existing_ids.contains(candidate.as_str()) {
            return candidate;
        }
        index += 1;
    }
}
