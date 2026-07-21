//! AI 查询数据结构：提示选项、Gemini 返回模型及到 `WordBankEntry` 的转换。

use serde::Deserialize;

use crate::structures::field_meta::field_label;
use crate::structures::word_bank_entry::{PartOfSpeech, UiLanguage, WordBankEntry};
use crate::utils::i18n::tr;

use super::shared::{normalize_part_of_speech_enum, normalize_text_list, normalize_text_opt};

pub(crate) const WORD_FORM_HINT_OPTIONS: [&str; 43] = [
    "unknown",
    "noun-baseform",
    "noun_plural",
    "noun_singular_definite",
    "noun_plural_definite",
    "noun_singular_definite_genitive",
    "noun_plural_definite_genitive",
    "noun_singular_indefinite_genitive",
    "noun_plural_indefinite_genitive",
    "verb-baseform",
    "verb_present_tense",
    "verb_past_tense",
    "verb_imperative",
    "verb_present_participle",
    "verb_past_participle",
    "verb_passive_infinitive",
    "verb_passive_present",
    "verb_passive_past",
    "adjective-baseform",
    "adjective_feminine_form",
    "adjective_neuter_form",
    "adjective_plural_form",
    "adjective_comparative",
    "adjective_superlative_indefinite",
    "adjective_superlative_definite",
    "adverb-baseform",
    "adverb_comparative",
    "adverb_superlative",
    "pronoun-baseform",
    "pronoun_object",
    "pronoun_reflexive",
    "pronoun_plural_subject",
    "pronoun_plural_object",
    "pronoun_plural_reflexive",
    "determinative-baseform",
    "determinative_feminine_form",
    "determinative_neuter_form",
    "determinative_plural_form",
    "preposition-baseform",
    "conjunction-baseform",
    "subjunction-baseform",
    "interjection-baseform",
    "phrase-baseform",
];

/// 词形提示下拉的双语显示名。
pub(crate) fn form_hint_label(lang: UiLanguage, hint: &str) -> String {
    if hint == "unknown" {
        return tr(lang, "未知", "Unknown").to_string();
    }

    if let Some(pos_key) = hint.strip_suffix("-baseform") {
        if let Ok(pos) = PartOfSpeech::from_key(pos_key) {
            return format!(
                "{}_{}",
                pos.display_name(lang),
                field_label(lang, "base_form")
            );
        }
    }

    let label = field_label(lang, hint);
    if label != "unknown" {
        return label.to_string();
    }

    hint.to_string()
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct GeminiWordResult {
    pub part_of_speech: Option<String>,
    pub base_form: Option<String>,
    pub chinese: Vec<String>,
    pub english: Vec<String>,
    #[serde(alias = "present_tense")]
    pub verb_present_tense: Option<String>,
    #[serde(alias = "past_tense")]
    pub verb_past_tense: Option<String>,
    #[serde(alias = "imperative")]
    pub verb_imperative: Option<String>,
    pub verb_present_participle: Option<String>,
    pub verb_past_participle: Option<String>,
    pub verb_passive_infinitive: Option<String>,
    pub verb_passive_present: Option<String>,
    pub verb_passive_past: Option<String>,
    #[serde(alias = "plural")]
    pub noun_plural: Option<String>,
    #[serde(alias = "singular_definite")]
    pub noun_singular_definite: Option<String>,
    #[serde(alias = "plural_definite")]
    pub noun_plural_definite: Option<String>,
    pub noun_singular_definite_genitive: Option<String>,
    pub noun_plural_definite_genitive: Option<String>,
    pub noun_singular_indefinite_genitive: Option<String>,
    pub noun_plural_indefinite_genitive: Option<String>,
    pub adjective_feminine_form: Option<String>,
    #[serde(alias = "neuter_form")]
    pub adjective_neuter_form: Option<String>,
    #[serde(alias = "plural_form")]
    pub adjective_plural_form: Option<String>,
    pub adjective_comparative: Option<String>,
    pub adjective_superlative_indefinite: Option<String>,
    pub adjective_superlative_definite: Option<String>,
    pub pronoun_object: Option<String>,
    pub pronoun_reflexive: Option<String>,
    pub pronoun_plural_subject: Option<String>,
    pub pronoun_plural_object: Option<String>,
    pub pronoun_plural_reflexive: Option<String>,
    pub determinative_feminine_form: Option<String>,
    pub determinative_neuter_form: Option<String>,
    pub determinative_plural_form: Option<String>,
    pub adverb_comparative: Option<String>,
    pub adverb_superlative: Option<String>,
}

impl TryFrom<(&GeminiWordResult, &str)> for WordBankEntry {
    type Error = String;

    /// 将 AI 结果按 hint 归一化后映射到项目词条结构。
    fn try_from((value, hint): (&GeminiWordResult, &str)) -> Result<Self, Self::Error> {
        Ok(Self {
            id: String::new(),
            selected: true,
            part_of_speech: normalize_part_of_speech_enum(value.part_of_speech.clone(), hint),
            tags: Vec::new(),
            english: normalize_text_list(&value.english),
            chinese: normalize_text_list(&value.chinese),
            base_form: normalize_text_opt(value.base_form.clone()).unwrap_or_default(),
            verb_present_tense: normalize_text_opt(value.verb_present_tense.clone()),
            verb_past_tense: normalize_text_opt(value.verb_past_tense.clone()),
            verb_imperative: normalize_text_opt(value.verb_imperative.clone()),
            verb_present_participle: normalize_text_opt(value.verb_present_participle.clone()),
            verb_past_participle: normalize_text_opt(value.verb_past_participle.clone()),
            verb_passive_infinitive: normalize_text_opt(value.verb_passive_infinitive.clone()),
            verb_passive_present: normalize_text_opt(value.verb_passive_present.clone()),
            verb_passive_past: normalize_text_opt(value.verb_passive_past.clone()),
            noun_plural: normalize_text_opt(value.noun_plural.clone()),
            noun_singular_definite: normalize_text_opt(value.noun_singular_definite.clone()),
            noun_plural_definite: normalize_text_opt(value.noun_plural_definite.clone()),
            noun_singular_definite_genitive: normalize_text_opt(
                value.noun_singular_definite_genitive.clone(),
            ),
            noun_plural_definite_genitive: normalize_text_opt(
                value.noun_plural_definite_genitive.clone(),
            ),
            noun_singular_indefinite_genitive: normalize_text_opt(
                value.noun_singular_indefinite_genitive.clone(),
            ),
            noun_plural_indefinite_genitive: normalize_text_opt(
                value.noun_plural_indefinite_genitive.clone(),
            ),
            adjective_feminine_form: normalize_text_opt(value.adjective_feminine_form.clone()),
            adjective_neuter_form: normalize_text_opt(value.adjective_neuter_form.clone()),
            adjective_plural_form: normalize_text_opt(value.adjective_plural_form.clone()),
            adjective_comparative: normalize_text_opt(value.adjective_comparative.clone()),
            adjective_superlative_indefinite: normalize_text_opt(
                value.adjective_superlative_indefinite.clone(),
            ),
            adjective_superlative_definite: normalize_text_opt(
                value.adjective_superlative_definite.clone(),
            ),
            pronoun_object: normalize_text_opt(value.pronoun_object.clone()),
            pronoun_reflexive: normalize_text_opt(value.pronoun_reflexive.clone()),
            pronoun_plural_subject: normalize_text_opt(value.pronoun_plural_subject.clone()),
            pronoun_plural_object: normalize_text_opt(value.pronoun_plural_object.clone()),
            pronoun_plural_reflexive: normalize_text_opt(value.pronoun_plural_reflexive.clone()),
            determinative_feminine_form: normalize_text_opt(
                value.determinative_feminine_form.clone(),
            ),
            determinative_neuter_form: normalize_text_opt(value.determinative_neuter_form.clone()),
            determinative_plural_form: normalize_text_opt(value.determinative_plural_form.clone()),
            adverb_comparative: normalize_text_opt(value.adverb_comparative.clone()),
            adverb_superlative: normalize_text_opt(value.adverb_superlative.clone()),
        })
    }
}
