use serde::Deserialize;

use crate::structures::word_bank_entry::WordBankEntry;

use super::shared::{normalize_part_of_speech_enum, normalize_text_list, normalize_text_opt};

pub(crate) const WORD_FORM_HINT_OPTIONS: [(&str, &str); 23] = [
    ("unknown", "未知"),
    ("noun-baseform", "noun-baseform"),
    ("noun_plural", "noun_plural"),
    ("noun_singular_definite", "noun_singular_definite"),
    ("noun_plural_definite", "noun_plural_definite"),
    ("verb-baseform", "verb-baseform"),
    ("verb_present_tense", "verb_present_tense"),
    ("verb_past_tense", "verb_past_tense"),
    ("verb_imperative", "verb_imperative"),
    ("adjective-baseform", "adjective-baseform"),
    ("adjective_neuter_form", "adjective_neuter_form"),
    ("adjective_plural_form", "adjective_plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    (
        "adjective_superlative_definite",
        "adjective_superlative_definite",
    ),
    ("adverb-baseform", "adverb-baseform"),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
    ("cardinal_number-baseform", "cardinal_number-baseform"),
    ("ordinal_number-baseform", "ordinal_number-baseform"),
    ("month-baseform", "month-baseform"),
    ("pronoun-baseform", "pronoun-baseform"),
    ("interrogative-baseform", "interrogative-baseform"),
];

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
    #[serde(alias = "plural")]
    pub noun_plural: Option<String>,
    #[serde(alias = "singular_definite")]
    pub noun_singular_definite: Option<String>,
    #[serde(alias = "plural_definite")]
    pub noun_plural_definite: Option<String>,
    #[serde(alias = "neuter_form")]
    pub adjective_neuter_form: Option<String>,
    #[serde(alias = "plural_form")]
    pub adjective_plural_form: Option<String>,
    pub adjective_comparative: Option<String>,
    pub adjective_superlative_indefinite: Option<String>,
    pub adjective_superlative_definite: Option<String>,
    pub adverb_comparative: Option<String>,
    pub adverb_superlative: Option<String>,
}

impl TryFrom<(&GeminiWordResult, &str)> for WordBankEntry {
    type Error = String;

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
            noun_plural: normalize_text_opt(value.noun_plural.clone()),
            noun_singular_definite: normalize_text_opt(value.noun_singular_definite.clone()),
            noun_plural_definite: normalize_text_opt(value.noun_plural_definite.clone()),
            adjective_neuter_form: normalize_text_opt(value.adjective_neuter_form.clone()),
            adjective_plural_form: normalize_text_opt(value.adjective_plural_form.clone()),
            adjective_comparative: normalize_text_opt(value.adjective_comparative.clone()),
            adjective_superlative_indefinite: normalize_text_opt(
                value.adjective_superlative_indefinite.clone(),
            ),
            adjective_superlative_definite: normalize_text_opt(
                value.adjective_superlative_definite.clone(),
            ),
            adverb_comparative: normalize_text_opt(value.adverb_comparative.clone()),
            adverb_superlative: normalize_text_opt(value.adverb_superlative.clone()),
        })
    }
}
