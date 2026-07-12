use serde::Deserialize;

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexiconBrowserMode {
    Edit,
    Query,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CsvWordEntry {
    pub(super) id: String,
    pub(super) selected: bool,
    pub(super) part_of_speech: String,
    pub(super) tags: String,
    pub(super) english: String,
    pub(super) chinese: String,
    pub(super) norwegian_base: String,
    pub(super) past_tense: String,
    pub(super) imperative: String,
    pub(super) plural: String,
    pub(super) singular_definite: String,
    pub(super) plural_definite: String,
    pub(super) neuter_form: String,
    pub(super) plural_form: String,
    pub(super) adjective_comparative: String,
    pub(super) adjective_superlative_indefinite: String,
    pub(super) adjective_superlative_definite: String,
    pub(super) adverb_comparative: String,
    pub(super) adverb_superlative: String,
}

impl TryFrom<CsvWordEntry> for WordBankEntry {
    type Error = String;

    fn try_from(value: CsvWordEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            selected: value.selected,
            part_of_speech: PartOfSpeech::from_key(&value.part_of_speech)?,
            tags: parse_pipe_list(&value.tags),
            english: parse_pipe_list(&value.english),
            chinese: parse_pipe_list(&value.chinese),
            base_form: value.norwegian_base,
            past_tense: parse_optional_text(&value.past_tense),
            imperative: parse_optional_text(&value.imperative),
            plural: parse_optional_text(&value.plural),
            singular_definite: parse_optional_text(&value.singular_definite),
            plural_definite: parse_optional_text(&value.plural_definite),
            neuter_form: parse_optional_text(&value.neuter_form),
            plural_form: parse_optional_text(&value.plural_form),
            adjective_comparative: parse_optional_text(&value.adjective_comparative),
            adjective_superlative_indefinite: parse_optional_text(
                &value.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: parse_optional_text(
                &value.adjective_superlative_definite,
            ),
            adverb_comparative: parse_optional_text(&value.adverb_comparative),
            adverb_superlative: parse_optional_text(&value.adverb_superlative),
        })
    }
}

impl From<&WordBankEntry> for CsvWordEntry {
    fn from(value: &WordBankEntry) -> Self {
        Self {
            id: value.id.clone(),
            selected: value.selected,
            part_of_speech: value.part_of_speech.as_key().to_string(),
            tags: value.tags.join("|"),
            english: value.english.join("|"),
            chinese: value.chinese.join("|"),
            norwegian_base: value.base_form.clone(),
            past_tense: optional_to_text(&value.past_tense),
            imperative: optional_to_text(&value.imperative),
            plural: optional_to_text(&value.plural),
            singular_definite: optional_to_text(&value.singular_definite),
            plural_definite: optional_to_text(&value.plural_definite),
            neuter_form: optional_to_text(&value.neuter_form),
            plural_form: optional_to_text(&value.plural_form),
            adjective_comparative: optional_to_text(&value.adjective_comparative),
            adjective_superlative_indefinite: optional_to_text(
                &value.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: optional_to_text(&value.adjective_superlative_definite),
            adverb_comparative: optional_to_text(&value.adverb_comparative),
            adverb_superlative: optional_to_text(&value.adverb_superlative),
        }
    }
}

fn parse_pipe_list(raw: &str) -> Vec<String> {
    raw.split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_optional_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn optional_to_text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}
