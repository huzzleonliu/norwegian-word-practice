//! 词库 CSV schema 工具：集中管理 CSV 结构、版本号、解析与序列化逻辑。

use std::collections::HashMap;

use serde::Deserialize;

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::dictionary::compute_word_entry_id;

pub const WORD_BANK_CSV_SCHEMA_VERSION: u16 = 1;
const SCHEMA_VERSION_PREFIX: &str = "#schema_version=";

#[derive(Clone, Debug, Deserialize)]
struct CsvWordEntry {
    id: String,
    selected: bool,
    part_of_speech: String,
    tags: String,
    english: String,
    chinese: String,
    norwegian_base: String,
    #[serde(default)]
    verb_present_tense: String,
    #[serde(alias = "past_tense")]
    verb_past_tense: String,
    #[serde(alias = "imperative")]
    verb_imperative: String,
    #[serde(default)]
    verb_present_participle: String,
    #[serde(default)]
    verb_past_participle: String,
    #[serde(default)]
    verb_passive_infinitive: String,
    #[serde(default)]
    verb_passive_present: String,
    #[serde(default)]
    verb_passive_past: String,
    #[serde(alias = "plural")]
    noun_plural: String,
    #[serde(alias = "singular_definite")]
    noun_singular_definite: String,
    #[serde(alias = "plural_definite")]
    noun_plural_definite: String,
    #[serde(default)]
    noun_singular_definite_genitive: String,
    #[serde(default)]
    noun_plural_definite_genitive: String,
    #[serde(default)]
    noun_singular_indefinite_genitive: String,
    #[serde(default)]
    noun_plural_indefinite_genitive: String,
    #[serde(default)]
    adjective_feminine_form: String,
    #[serde(alias = "neuter_form")]
    adjective_neuter_form: String,
    #[serde(alias = "plural_form")]
    adjective_plural_form: String,
    adjective_comparative: String,
    adjective_superlative_indefinite: String,
    adjective_superlative_definite: String,
    #[serde(default)]
    pronoun_object: String,
    #[serde(default)]
    pronoun_reflexive: String,
    #[serde(default)]
    pronoun_plural_subject: String,
    #[serde(default)]
    pronoun_plural_object: String,
    #[serde(default)]
    pronoun_plural_reflexive: String,
    #[serde(default)]
    determinative_feminine_form: String,
    #[serde(default)]
    determinative_neuter_form: String,
    #[serde(default)]
    determinative_plural_form: String,
    adverb_comparative: String,
    adverb_superlative: String,
}

impl TryFrom<CsvWordEntry> for WordBankEntry {
    type Error = String;

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

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordBankEntry>, String> {
    let (schema_version, body) = split_schema_version(content)?;
    if let Some(version) = schema_version {
        if version > WORD_BANK_CSV_SCHEMA_VERSION {
            return Err(format!(
                "CSV schema 版本过高：{version}（当前支持 <= {}）",
                WORD_BANK_CSV_SCHEMA_VERSION
            ));
        }
    }

    let has_header = detect_word_bank_header(body);
    match parse_word_bank_csv_inner(body, has_header) {
        Ok(entries) => Ok(entries),
        Err(primary_err) => parse_word_bank_csv_inner(body, !has_header).map_err(|_| primary_err),
    }
}

pub fn serialize_word_bank_csv(entries: &[WordBankEntry]) -> Result<String, String> {
    let mut writer = csv::Writer::from_writer(Vec::<u8>::new());
    writer
        .write_record([
            "id",
            "selected",
            "part_of_speech",
            "tags",
            "english",
            "chinese",
            "norwegian_base",
            "verb_present_tense",
            "verb_past_tense",
            "verb_imperative",
            "noun_plural",
            "noun_singular_definite",
            "noun_plural_definite",
            "adjective_neuter_form",
            "adjective_plural_form",
            "adjective_comparative",
            "adjective_superlative_indefinite",
            "adjective_superlative_definite",
            "adverb_comparative",
            "adverb_superlative",
            "verb_present_participle",
            "verb_past_participle",
            "verb_passive_infinitive",
            "verb_passive_present",
            "verb_passive_past",
            "noun_singular_definite_genitive",
            "noun_plural_definite_genitive",
            "noun_singular_indefinite_genitive",
            "noun_plural_indefinite_genitive",
            "adjective_feminine_form",
            "pronoun_object",
            "pronoun_reflexive",
            "pronoun_plural_subject",
            "pronoun_plural_object",
            "pronoun_plural_reflexive",
            "determinative_feminine_form",
            "determinative_neuter_form",
            "determinative_plural_form",
        ])
        .map_err(|err| format!("CSV 写入失败: {err}"))?;

    for entry in entries {
        let csv_entry = CsvWordEntry::from(entry);
        writer
            .write_record([
                csv_entry.id.as_str(),
                if csv_entry.selected { "true" } else { "false" },
                csv_entry.part_of_speech.as_str(),
                csv_entry.tags.as_str(),
                csv_entry.english.as_str(),
                csv_entry.chinese.as_str(),
                csv_entry.norwegian_base.as_str(),
                csv_entry.verb_present_tense.as_str(),
                csv_entry.verb_past_tense.as_str(),
                csv_entry.verb_imperative.as_str(),
                csv_entry.noun_plural.as_str(),
                csv_entry.noun_singular_definite.as_str(),
                csv_entry.noun_plural_definite.as_str(),
                csv_entry.adjective_neuter_form.as_str(),
                csv_entry.adjective_plural_form.as_str(),
                csv_entry.adjective_comparative.as_str(),
                csv_entry.adjective_superlative_indefinite.as_str(),
                csv_entry.adjective_superlative_definite.as_str(),
                csv_entry.adverb_comparative.as_str(),
                csv_entry.adverb_superlative.as_str(),
                csv_entry.verb_present_participle.as_str(),
                csv_entry.verb_past_participle.as_str(),
                csv_entry.verb_passive_infinitive.as_str(),
                csv_entry.verb_passive_present.as_str(),
                csv_entry.verb_passive_past.as_str(),
                csv_entry.noun_singular_definite_genitive.as_str(),
                csv_entry.noun_plural_definite_genitive.as_str(),
                csv_entry.noun_singular_indefinite_genitive.as_str(),
                csv_entry.noun_plural_indefinite_genitive.as_str(),
                csv_entry.adjective_feminine_form.as_str(),
                csv_entry.pronoun_object.as_str(),
                csv_entry.pronoun_reflexive.as_str(),
                csv_entry.pronoun_plural_subject.as_str(),
                csv_entry.pronoun_plural_object.as_str(),
                csv_entry.pronoun_plural_reflexive.as_str(),
                csv_entry.determinative_feminine_form.as_str(),
                csv_entry.determinative_neuter_form.as_str(),
                csv_entry.determinative_plural_form.as_str(),
            ])
            .map_err(|err| format!("CSV 写入失败: {err}"))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|err| format!("CSV 生成失败: {}", err.error()))?;
    let csv_body = String::from_utf8(bytes).map_err(|err| format!("CSV UTF-8 转换失败: {err}"))?;
    Ok(format!(
        "{SCHEMA_VERSION_PREFIX}{WORD_BANK_CSV_SCHEMA_VERSION}\n{csv_body}"
    ))
}

fn split_schema_version(content: &str) -> Result<(Option<u16>, &str), String> {
    let normalized = content.trim_start_matches('\u{feff}');
    let Some(first_line_end) = normalized.find('\n') else {
        let first_line = normalized.trim_end_matches('\r').trim();
        if let Some(version_text) = first_line.strip_prefix(SCHEMA_VERSION_PREFIX) {
            let version = version_text
                .trim()
                .parse::<u16>()
                .map_err(|_| format!("无效 schema version: {version_text}"))?;
            return Ok((Some(version), ""));
        }
        return Ok((None, normalized));
    };

    let first_line = normalized[..first_line_end].trim_end_matches('\r').trim();
    if let Some(version_text) = first_line.strip_prefix(SCHEMA_VERSION_PREFIX) {
        let version = version_text
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("无效 schema version: {version_text}"))?;
        Ok((Some(version), &normalized[first_line_end + 1..]))
    } else {
        Ok((None, normalized))
    }
}

fn detect_word_bank_header(content: &str) -> bool {
    let Some(first_non_empty_line) = content.lines().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let normalized = first_non_empty_line
        .trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase();
    normalized.starts_with("id,selected,part_of_speech,")
}

fn parse_word_bank_csv_inner(
    content: &str,
    has_headers: bool,
) -> Result<Vec<WordBankEntry>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut entries = Vec::new();
    let mut seen_entry_rows = HashMap::<String, usize>::new();
    for (row_index, row) in reader.deserialize::<CsvWordEntry>().enumerate() {
        let row = row.map_err(|err| format!("CSV 解析失败: {err}"))?;
        let entry = WordBankEntry::try_from(row)?;
        let current_line = if has_headers {
            row_index + 2
        } else {
            row_index + 1
        };
        if let Some(first_line) = seen_entry_rows.insert(entry.id.clone(), current_line) {
            return Err(format!(
                "CSV 存在重复词条（同词性+词形组合），首行 {first_line} 与第 {current_line} 行冲突。"
            ));
        }
        entries.push(entry);
    }

    Ok(entries)
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
