//! 词库浏览器结构定义：浏览模式枚举与 CSV 行结构及其双向转换。

use serde::Deserialize;

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::dictionary::compute_word_entry_id;

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
    #[serde(default)]
    pub(super) verb_present_tense: String,
    #[serde(alias = "past_tense")]
    pub(super) verb_past_tense: String,
    #[serde(alias = "imperative")]
    pub(super) verb_imperative: String,
    #[serde(default)]
    pub(super) verb_present_participle: String,
    #[serde(default)]
    pub(super) verb_past_participle: String,
    #[serde(default)]
    pub(super) verb_passive_infinitive: String,
    #[serde(default)]
    pub(super) verb_passive_present: String,
    #[serde(default)]
    pub(super) verb_passive_past: String,
    #[serde(alias = "plural")]
    pub(super) noun_plural: String,
    #[serde(alias = "singular_definite")]
    pub(super) noun_singular_definite: String,
    #[serde(alias = "plural_definite")]
    pub(super) noun_plural_definite: String,
    #[serde(default)]
    pub(super) noun_singular_definite_genitive: String,
    #[serde(default)]
    pub(super) noun_plural_definite_genitive: String,
    #[serde(default)]
    pub(super) noun_singular_indefinite_genitive: String,
    #[serde(default)]
    pub(super) noun_plural_indefinite_genitive: String,
    #[serde(default)]
    pub(super) adjective_feminine_form: String,
    #[serde(alias = "neuter_form")]
    pub(super) adjective_neuter_form: String,
    #[serde(alias = "plural_form")]
    pub(super) adjective_plural_form: String,
    pub(super) adjective_comparative: String,
    pub(super) adjective_superlative_indefinite: String,
    pub(super) adjective_superlative_definite: String,
    #[serde(default)]
    pub(super) pronoun_object: String,
    #[serde(default)]
    pub(super) pronoun_reflexive: String,
    #[serde(default)]
    pub(super) pronoun_plural_subject: String,
    #[serde(default)]
    pub(super) pronoun_plural_object: String,
    #[serde(default)]
    pub(super) pronoun_plural_reflexive: String,
    #[serde(default)]
    pub(super) determinative_feminine_form: String,
    #[serde(default)]
    pub(super) determinative_neuter_form: String,
    #[serde(default)]
    pub(super) determinative_plural_form: String,
    pub(super) adverb_comparative: String,
    pub(super) adverb_superlative: String,
}

impl TryFrom<CsvWordEntry> for WordBankEntry {
    type Error = String;

    /// CSV 行 -> 词条结构：
    /// - 解析词性
    /// - 规范化可选字段
    /// - 根据词形重算稳定 id
    fn try_from(value: CsvWordEntry) -> Result<Self, Self::Error> {
        let mut entry = Self {
            id: String::new(),
            selected: value.selected,
            part_of_speech: PartOfSpeech::from_key(&value.part_of_speech)?,
            tags: parse_pipe_list(&value.tags),
            english: parse_pipe_list(&value.english),
            chinese: parse_pipe_list(&value.chinese),
            base_form: value.norwegian_base,
            verb_present_tense: parse_optional_text(&value.verb_present_tense),
            verb_past_tense: parse_optional_text(&value.verb_past_tense),
            verb_imperative: parse_optional_text(&value.verb_imperative),
            verb_present_participle: parse_optional_text(&value.verb_present_participle),
            verb_past_participle: parse_optional_text(&value.verb_past_participle),
            verb_passive_infinitive: parse_optional_text(&value.verb_passive_infinitive),
            verb_passive_present: parse_optional_text(&value.verb_passive_present),
            verb_passive_past: parse_optional_text(&value.verb_passive_past),
            noun_plural: parse_optional_text(&value.noun_plural),
            noun_singular_definite: parse_optional_text(&value.noun_singular_definite),
            noun_plural_definite: parse_optional_text(&value.noun_plural_definite),
            noun_singular_definite_genitive: parse_optional_text(
                &value.noun_singular_definite_genitive,
            ),
            noun_plural_definite_genitive: parse_optional_text(
                &value.noun_plural_definite_genitive,
            ),
            noun_singular_indefinite_genitive: parse_optional_text(
                &value.noun_singular_indefinite_genitive,
            ),
            noun_plural_indefinite_genitive: parse_optional_text(
                &value.noun_plural_indefinite_genitive,
            ),
            adjective_feminine_form: parse_optional_text(&value.adjective_feminine_form),
            adjective_neuter_form: parse_optional_text(&value.adjective_neuter_form),
            adjective_plural_form: parse_optional_text(&value.adjective_plural_form),
            adjective_comparative: parse_optional_text(&value.adjective_comparative),
            adjective_superlative_indefinite: parse_optional_text(
                &value.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: parse_optional_text(
                &value.adjective_superlative_definite,
            ),
            pronoun_object: parse_optional_text(&value.pronoun_object),
            pronoun_reflexive: parse_optional_text(&value.pronoun_reflexive),
            pronoun_plural_subject: parse_optional_text(&value.pronoun_plural_subject),
            pronoun_plural_object: parse_optional_text(&value.pronoun_plural_object),
            pronoun_plural_reflexive: parse_optional_text(&value.pronoun_plural_reflexive),
            determinative_feminine_form: parse_optional_text(&value.determinative_feminine_form),
            determinative_neuter_form: parse_optional_text(&value.determinative_neuter_form),
            determinative_plural_form: parse_optional_text(&value.determinative_plural_form),
            adverb_comparative: parse_optional_text(&value.adverb_comparative),
            adverb_superlative: parse_optional_text(&value.adverb_superlative),
        };
        entry.id = compute_word_entry_id(&entry);
        Ok(entry)
    }
}

impl From<&WordBankEntry> for CsvWordEntry {
    /// 词条结构 -> CSV 行（导出时始终重新计算 id）。
    fn from(value: &WordBankEntry) -> Self {
        Self {
            id: compute_word_entry_id(value),
            selected: value.selected,
            part_of_speech: value.part_of_speech.as_key().to_string(),
            tags: value.tags.join("|"),
            english: value.english.join("|"),
            chinese: value.chinese.join("|"),
            norwegian_base: value.base_form.clone(),
            verb_present_tense: optional_to_text(&value.verb_present_tense),
            verb_past_tense: optional_to_text(&value.verb_past_tense),
            verb_imperative: optional_to_text(&value.verb_imperative),
            verb_present_participle: optional_to_text(&value.verb_present_participle),
            verb_past_participle: optional_to_text(&value.verb_past_participle),
            verb_passive_infinitive: optional_to_text(&value.verb_passive_infinitive),
            verb_passive_present: optional_to_text(&value.verb_passive_present),
            verb_passive_past: optional_to_text(&value.verb_passive_past),
            noun_plural: optional_to_text(&value.noun_plural),
            noun_singular_definite: optional_to_text(&value.noun_singular_definite),
            noun_plural_definite: optional_to_text(&value.noun_plural_definite),
            noun_singular_definite_genitive: optional_to_text(
                &value.noun_singular_definite_genitive,
            ),
            noun_plural_definite_genitive: optional_to_text(&value.noun_plural_definite_genitive),
            noun_singular_indefinite_genitive: optional_to_text(
                &value.noun_singular_indefinite_genitive,
            ),
            noun_plural_indefinite_genitive: optional_to_text(
                &value.noun_plural_indefinite_genitive,
            ),
            adjective_feminine_form: optional_to_text(&value.adjective_feminine_form),
            adjective_neuter_form: optional_to_text(&value.adjective_neuter_form),
            adjective_plural_form: optional_to_text(&value.adjective_plural_form),
            adjective_comparative: optional_to_text(&value.adjective_comparative),
            adjective_superlative_indefinite: optional_to_text(
                &value.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: optional_to_text(&value.adjective_superlative_definite),
            pronoun_object: optional_to_text(&value.pronoun_object),
            pronoun_reflexive: optional_to_text(&value.pronoun_reflexive),
            pronoun_plural_subject: optional_to_text(&value.pronoun_plural_subject),
            pronoun_plural_object: optional_to_text(&value.pronoun_plural_object),
            pronoun_plural_reflexive: optional_to_text(&value.pronoun_plural_reflexive),
            determinative_feminine_form: optional_to_text(&value.determinative_feminine_form),
            determinative_neuter_form: optional_to_text(&value.determinative_neuter_form),
            determinative_plural_form: optional_to_text(&value.determinative_plural_form),
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
