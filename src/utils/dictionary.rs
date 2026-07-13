use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech, WordBankEntry};

pub type SingleEntryDraft = WordBankEntry;

/// 词性校验中不要求填写、使用频率较低的变体字段；UI 默认不勾选。
pub const OPTIONAL_VARIANT_FIELD_KEYS: [&str; 5] = [
    "verb_passive_past",
    "noun_singular_indefinite_genitive",
    "noun_plural_indefinite_genitive",
    "adverb_comparative",
    "adverb_superlative",
];

pub fn is_optional_variant_field(key: &str) -> bool {
    OPTIONAL_VARIANT_FIELD_KEYS.contains(&key)
}

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

/// 批量校验并规范化词库条目。每条仅计算一次哈希，重复检测为 O(n)。
#[allow(dead_code)]
pub fn validate_and_prepare_all_entries(
    entries: &[WordBankEntry],
) -> Result<Vec<WordBankEntry>, Vec<String>> {
    let mut validated = Vec::with_capacity(entries.len());
    let mut errors = Vec::new();
    let mut id_to_line: HashMap<String, usize> = HashMap::new();

    for (idx, item) in entries.iter().enumerate() {
        let line = idx + 1;
        match prepare_entry_for_storage(draft_from_word_entry(item)) {
            Ok(prepared) => {
                if let Some(&dup_line) = id_to_line.get(&prepared.id) {
                    errors.push(format!(
                        "第 {line} 行与第 {dup_line} 行重复（id: {}）：词性 `{}` + 词形组合已存在（原型 `{}`）。",
                        prepared.id,
                        prepared.part_of_speech.as_key(),
                        prepared.base_form,
                    ));
                } else {
                    id_to_line.insert(prepared.id.clone(), line);
                    validated.push(prepared);
                }
            }
            Err(err) => errors.push(format!("第 {line} 行（id: {}）{err}", item.id)),
        }
    }

    if errors.is_empty() {
        Ok(validated)
    } else {
        Err(errors)
    }
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
            require_option("现在分词", &draft.verb_present_participle)?;
            require_option("过去分词", &draft.verb_past_participle)?;
            require_option("被动不定式", &draft.verb_passive_infinitive)?;
            require_option("被动现在时", &draft.verb_passive_present)?;
        }
        PartOfSpeech::Noun => {
            require_option("复数", &draft.noun_plural)?;
            require_option("单数特指", &draft.noun_singular_definite)?;
            require_option("复数特指", &draft.noun_plural_definite)?;
            require_option("单数特指所有格", &draft.noun_singular_definite_genitive)?;
            require_option("复数特指所有格", &draft.noun_plural_definite_genitive)?;
        }
        PartOfSpeech::Adjective => {
            require_option("阴性形式", &draft.adjective_feminine_form)?;
            require_option("中性形式", &draft.adjective_neuter_form)?;
            require_option("复数形式", &draft.adjective_plural_form)?;
            require_option("比较级", &draft.adjective_comparative)?;
            require_option("最高级泛指", &draft.adjective_superlative_indefinite)?;
            require_option("最高级特指", &draft.adjective_superlative_definite)?;
        }
        PartOfSpeech::Pronoun => {
            require_option("宾格形式", &draft.pronoun_object)?;
            require_option("反身形式", &draft.pronoun_reflexive)?;
            require_option("复数主格", &draft.pronoun_plural_subject)?;
            require_option("复数宾格", &draft.pronoun_plural_object)?;
            require_option("复数反身", &draft.pronoun_plural_reflexive)?;
        }
        PartOfSpeech::Determinative => {
            require_option("限定词阴性", &draft.determinative_feminine_form)?;
            require_option("限定词中性", &draft.determinative_neuter_form)?;
            require_option("限定词复数", &draft.determinative_plural_form)?;
        }
        PartOfSpeech::Adverb
        | PartOfSpeech::Preposition
        | PartOfSpeech::Conjunction
        | PartOfSpeech::Subjunction
        | PartOfSpeech::Interjection => {}
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
        verb_present_participle: normalize_optional(draft.verb_present_participle),
        verb_past_participle: normalize_optional(draft.verb_past_participle),
        verb_passive_infinitive: normalize_optional(draft.verb_passive_infinitive),
        verb_passive_present: normalize_optional(draft.verb_passive_present),
        verb_passive_past: normalize_optional(draft.verb_passive_past),
        noun_plural: normalize_optional(draft.noun_plural),
        noun_singular_definite: normalize_optional(draft.noun_singular_definite),
        noun_plural_definite: normalize_optional(draft.noun_plural_definite),
        noun_singular_definite_genitive: normalize_optional(draft.noun_singular_definite_genitive),
        noun_plural_definite_genitive: normalize_optional(draft.noun_plural_definite_genitive),
        noun_singular_indefinite_genitive: normalize_optional(
            draft.noun_singular_indefinite_genitive,
        ),
        noun_plural_indefinite_genitive: normalize_optional(draft.noun_plural_indefinite_genitive),
        adjective_feminine_form: normalize_optional(draft.adjective_feminine_form),
        adjective_neuter_form: normalize_optional(draft.adjective_neuter_form),
        adjective_plural_form: normalize_optional(draft.adjective_plural_form),
        adjective_comparative: normalize_optional(draft.adjective_comparative),
        adjective_superlative_indefinite: normalize_optional(
            draft.adjective_superlative_indefinite,
        ),
        adjective_superlative_definite: normalize_optional(draft.adjective_superlative_definite),
        pronoun_object: normalize_optional(draft.pronoun_object),
        pronoun_reflexive: normalize_optional(draft.pronoun_reflexive),
        pronoun_plural_subject: normalize_optional(draft.pronoun_plural_subject),
        pronoun_plural_object: normalize_optional(draft.pronoun_plural_object),
        pronoun_plural_reflexive: normalize_optional(draft.pronoun_plural_reflexive),
        determinative_feminine_form: normalize_optional(draft.determinative_feminine_form),
        determinative_neuter_form: normalize_optional(draft.determinative_neuter_form),
        determinative_plural_form: normalize_optional(draft.determinative_plural_form),
        adverb_comparative: normalize_optional(draft.adverb_comparative),
        adverb_superlative: normalize_optional(draft.adverb_superlative),
    };
    entry.id = compute_word_entry_id(&entry);
    Ok(entry)
}

pub fn compute_word_entry_id(entry: &WordBankEntry) -> String {
    let payload = format!(
        "v2|pos={}|base={}|verb_present={}|verb_past={}|verb_imperative={}|verb_present_participle={}|verb_past_participle={}|verb_passive_infinitive={}|verb_passive_present={}|verb_passive_past={}|noun_plural={}|noun_singular_definite={}|noun_plural_definite={}|noun_singular_definite_genitive={}|noun_plural_definite_genitive={}|noun_singular_indefinite_genitive={}|noun_plural_indefinite_genitive={}|adjective_feminine_form={}|adjective_neuter_form={}|adjective_plural_form={}|adjective_comparative={}|adjective_superlative_indefinite={}|adjective_superlative_definite={}|pronoun_object={}|pronoun_reflexive={}|pronoun_plural_subject={}|pronoun_plural_object={}|pronoun_plural_reflexive={}|determinative_feminine_form={}|determinative_neuter_form={}|determinative_plural_form={}|adverb_comparative={}|adverb_superlative={}",
        normalize_for_hash(entry.part_of_speech.as_key()),
        normalize_for_hash(&entry.base_form),
        normalize_option_for_hash(&entry.verb_present_tense),
        normalize_option_for_hash(&entry.verb_past_tense),
        normalize_option_for_hash(&entry.verb_imperative),
        normalize_option_for_hash(&entry.verb_present_participle),
        normalize_option_for_hash(&entry.verb_past_participle),
        normalize_option_for_hash(&entry.verb_passive_infinitive),
        normalize_option_for_hash(&entry.verb_passive_present),
        normalize_option_for_hash(&entry.verb_passive_past),
        normalize_option_for_hash(&entry.noun_plural),
        normalize_option_for_hash(&entry.noun_singular_definite),
        normalize_option_for_hash(&entry.noun_plural_definite),
        normalize_option_for_hash(&entry.noun_singular_definite_genitive),
        normalize_option_for_hash(&entry.noun_plural_definite_genitive),
        normalize_option_for_hash(&entry.noun_singular_indefinite_genitive),
        normalize_option_for_hash(&entry.noun_plural_indefinite_genitive),
        normalize_option_for_hash(&entry.adjective_feminine_form),
        normalize_option_for_hash(&entry.adjective_neuter_form),
        normalize_option_for_hash(&entry.adjective_plural_form),
        normalize_option_for_hash(&entry.adjective_comparative),
        normalize_option_for_hash(&entry.adjective_superlative_indefinite),
        normalize_option_for_hash(&entry.adjective_superlative_definite),
        normalize_option_for_hash(&entry.pronoun_object),
        normalize_option_for_hash(&entry.pronoun_reflexive),
        normalize_option_for_hash(&entry.pronoun_plural_subject),
        normalize_option_for_hash(&entry.pronoun_plural_object),
        normalize_option_for_hash(&entry.pronoun_plural_reflexive),
        normalize_option_for_hash(&entry.determinative_feminine_form),
        normalize_option_for_hash(&entry.determinative_neuter_form),
        normalize_option_for_hash(&entry.determinative_plural_form),
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
