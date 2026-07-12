use sha2::{Digest, Sha256};

use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech, WordBankEntry};

pub type SingleEntryDraft = WordBankEntry;

pub fn parse_part_of_speech(raw: &str) -> Result<PartOfSpeech, String> {
    PartOfSpeech::from_key(raw).map_err(|_| {
        format!(
            "词性不合法，请从预设选项中选择：{}",
            PART_OF_SPEECH_OPTIONS.join(", ")
        )
    })
}

pub fn validate_and_prepare_single_entry(
    draft: SingleEntryDraft,
    existing_entries: &[WordBankEntry],
) -> Result<WordBankEntry, String> {
    let prepared_entry = prepare_entry_for_storage(draft)?;
    if existing_entries
        .iter()
        .any(|entry| compute_word_entry_id(entry) == prepared_entry.id)
    {
        return Err(format!(
            "词条重复：词性 `{}` + 词形组合已存在（原型 `{}`）。",
            prepared_entry.part_of_speech.as_key(),
            prepared_entry.base_form
        ));
    }

    Ok(prepared_entry)
}

pub fn validate_existing_entry(
    entry: &WordBankEntry,
    existing_entries: &[WordBankEntry],
    self_index: usize,
) -> Result<WordBankEntry, String> {
    let prepared_entry = prepare_entry_for_storage(draft_from_word_entry(entry))?;
    if existing_entries
        .iter()
        .enumerate()
        .any(|(idx, item)| idx != self_index && compute_word_entry_id(item) == prepared_entry.id)
    {
        return Err(format!(
            "词条重复：词性 `{}` + 词形组合已存在（原型 `{}`）。",
            prepared_entry.part_of_speech.as_key(),
            prepared_entry.base_form
        ));
    }

    Ok(prepared_entry)
}

pub fn draft_from_word_entry(entry: &WordBankEntry) -> SingleEntryDraft {
    entry.clone()
}

fn validate_forms_by_part_of_speech(
    pos: &PartOfSpeech,
    draft: &SingleEntryDraft,
) -> Result<(), String> {
    match pos {
        PartOfSpeech::Verb => {
            require_option("现在时", &draft.verb_present_tense)?;
            require_option("过去式", &draft.verb_past_tense)?;
            require_option("祈使式", &draft.verb_imperative)?;
        }
        PartOfSpeech::Noun => {
            require_option("复数", &draft.noun_plural)?;
            require_option("单数特指", &draft.noun_singular_definite)?;
            require_option("复数特指", &draft.noun_plural_definite)?;
        }
        PartOfSpeech::Adjective => {
            require_option("对应中性", &draft.adjective_neuter_form)?;
            require_option("对应复数", &draft.adjective_plural_form)?;
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

fn prepare_entry_for_storage(draft: SingleEntryDraft) -> Result<WordBankEntry, String> {
    let base_form = draft.base_form.trim().to_string();
    if base_form.is_empty() {
        return Err("原型不能为空。".to_string());
    }

    validate_forms_by_part_of_speech(&draft.part_of_speech, &draft)?;

    let chinese = normalize_vec(draft.chinese);
    if chinese.is_empty() {
        return Err("对应中文不能为空。".to_string());
    }

    let mut entry = WordBankEntry {
        id: String::new(),
        selected: draft.selected,
        part_of_speech: draft.part_of_speech,
        tags: normalize_vec(draft.tags),
        english: normalize_vec(draft.english),
        chinese,
        base_form,
        verb_present_tense: normalize_optional(draft.verb_present_tense),
        verb_past_tense: normalize_optional(draft.verb_past_tense),
        verb_imperative: normalize_optional(draft.verb_imperative),
        noun_plural: normalize_optional(draft.noun_plural),
        noun_singular_definite: normalize_optional(draft.noun_singular_definite),
        noun_plural_definite: normalize_optional(draft.noun_plural_definite),
        adjective_neuter_form: normalize_optional(draft.adjective_neuter_form),
        adjective_plural_form: normalize_optional(draft.adjective_plural_form),
        adjective_comparative: normalize_optional(draft.adjective_comparative),
        adjective_superlative_indefinite: normalize_optional(
            draft.adjective_superlative_indefinite,
        ),
        adjective_superlative_definite: normalize_optional(draft.adjective_superlative_definite),
        adverb_comparative: normalize_optional(draft.adverb_comparative),
        adverb_superlative: normalize_optional(draft.adverb_superlative),
    };
    entry.id = compute_word_entry_id(&entry);
    Ok(entry)
}

pub fn compute_word_entry_id(entry: &WordBankEntry) -> String {
    let payload = format!(
        "v1|pos={}|base={}|verb_present={}|verb_past={}|verb_imperative={}|noun_plural={}|noun_singular_definite={}|noun_plural_definite={}|adjective_neuter_form={}|adjective_plural_form={}|adjective_comparative={}|adjective_superlative_indefinite={}|adjective_superlative_definite={}|adverb_comparative={}|adverb_superlative={}",
        normalize_for_hash(entry.part_of_speech.as_key()),
        normalize_for_hash(&entry.base_form),
        normalize_option_for_hash(&entry.verb_present_tense),
        normalize_option_for_hash(&entry.verb_past_tense),
        normalize_option_for_hash(&entry.verb_imperative),
        normalize_option_for_hash(&entry.noun_plural),
        normalize_option_for_hash(&entry.noun_singular_definite),
        normalize_option_for_hash(&entry.noun_plural_definite),
        normalize_option_for_hash(&entry.adjective_neuter_form),
        normalize_option_for_hash(&entry.adjective_plural_form),
        normalize_option_for_hash(&entry.adjective_comparative),
        normalize_option_for_hash(&entry.adjective_superlative_indefinite),
        normalize_option_for_hash(&entry.adjective_superlative_definite),
        normalize_option_for_hash(&entry.adverb_comparative),
        normalize_option_for_hash(&entry.adverb_superlative),
    );

    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("wb_{hash_hex}")
}

fn normalize_for_hash(raw: &str) -> String {
    raw.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalize_option_for_hash(value: &Option<String>) -> String {
    value
        .as_ref()
        .map(|item| normalize_for_hash(item))
        .unwrap_or_default()
}
