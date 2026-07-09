use serde::{Deserialize, Serialize};

pub const PART_OF_SPEECH_OPTIONS: [&str; 9] = [
    "verb",
    "noun",
    "adjective",
    "adverb",
    "cardinal_number",
    "ordinal_number",
    "month",
    "pronoun",
    "interrogative",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WordEntry {
    pub id: String,
    pub selected: bool,
    pub part_of_speech: String,
    pub tags: Vec<String>,
    pub english: Vec<String>,
    pub chinese: Vec<String>,
    pub base_form: String,
    pub past_tense: Option<String>,
    pub imperative: Option<String>,
    pub plural: Option<String>,
    pub singular_definite: Option<String>,
    pub plural_definite: Option<String>,
    pub neuter_form: Option<String>,
    pub plural_form: Option<String>,
    pub adjective_comparative: Option<String>,
    pub adjective_superlative_indefinite: Option<String>,
    pub adjective_superlative_definite: Option<String>,
    pub adverb_comparative: Option<String>,
    pub adverb_superlative: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexiconBrowserMode {
    Edit,
    Query,
}

#[derive(Debug, Deserialize)]
pub(super) struct CsvWordEntry {
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
