use std::cmp::Ordering;
use std::collections::HashMap;

use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};
use crate::utils::dictionary::OPTIONAL_VARIANT_FIELD_KEYS;
use crate::utils::i18n::tr;

use super::structures::CsvWordEntry;

pub fn default_search_column_visibility() -> Vec<bool> {
    let mut cols = vec![true; DATA_COLUMN_KEYS.len()];
    if let Some(id_col) = cols.get_mut(0) {
        *id_col = false;
    }
    for key in OPTIONAL_VARIANT_FIELD_KEYS {
        if let Some(idx) = DATA_COLUMN_KEYS.iter().position(|&col_key| col_key == key) {
            if let Some(col) = cols.get_mut(idx) {
                *col = false;
            }
        }
    }
    cols
}

pub const DATA_COLUMN_KEYS: [&str; 38] = [
    "id",
    "selected",
    "part_of_speech",
    "tags",
    "english",
    "chinese",
    "base_form",
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
];

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordBankEntry>, String> {
    let has_header = detect_word_bank_header(content);
    match parse_word_bank_csv_inner(content, has_header) {
        Ok(entries) => Ok(entries),
        Err(primary_err) => {
            // Be tolerant to CSV exported by external tools (e.g. BOM/modified headers):
            // retry with the opposite header mode before failing.
            parse_word_bank_csv_inner(content, !has_header).map_err(|_| primary_err)
        }
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
    String::from_utf8(bytes).map_err(|err| format!("CSV UTF-8 转换失败: {err}"))
}

pub fn parse_pipe_list(raw: &str) -> Vec<String> {
    raw.split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn compare_entries_by_rules(
    a: &WordBankEntry,
    b: &WordBankEntry,
    rules: &[(usize, bool)],
) -> Ordering {
    for (col_idx, asc) in rules {
        let ordering = compare_entries_by_column(a, b, *col_idx, *asc);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.id.cmp(&b.id)
}

pub(super) fn entry_matches_filter(entry: &WordBankEntry, query: &str, columns: &[bool]) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let query_lower = trimmed.to_lowercase();

    (0..DATA_COLUMN_KEYS.len()).any(|idx| {
        columns.get(idx).copied().unwrap_or(false)
            && column_value_text(entry, idx)
                .to_lowercase()
                .contains(&query_lower)
    })
}

pub(super) fn header_name(col_idx: usize) -> &'static str {
    DATA_COLUMN_KEYS.get(col_idx).copied().unwrap_or("unknown")
}

pub(super) fn format_sort_rules(rules: &[(usize, bool)], lang: UiLanguage) -> String {
    if rules.is_empty() {
        return tr(lang, "未应用排序", "No sort applied").to_string();
    }

    rules
        .iter()
        .enumerate()
        .map(|(idx, (col, asc))| {
            format!(
                "{}. {}{}",
                idx + 1,
                header_name(*col),
                if *asc { "↑" } else { "↓" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn option_to_input(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

pub(super) fn input_to_option(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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

fn compare_entries_by_column(
    a: &WordBankEntry,
    b: &WordBankEntry,
    col_idx: usize,
    ascending: bool,
) -> Ordering {
    let ordering = column_value_text(a, col_idx).cmp(&column_value_text(b, col_idx));

    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

fn column_value_text(entry: &WordBankEntry, col_idx: usize) -> String {
    match col_idx {
        0 => entry.id.clone(),
        1 => {
            if entry.selected {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        2 => entry.part_of_speech.as_key().to_string(),
        3 => entry.tags.join("|"),
        4 => entry.english.join("|"),
        5 => entry.chinese.join("|"),
        6 => entry.base_form.clone(),
        7 => entry.verb_present_tense.clone().unwrap_or_default(),
        8 => entry.verb_past_tense.clone().unwrap_or_default(),
        9 => entry.verb_imperative.clone().unwrap_or_default(),
        10 => entry.noun_plural.clone().unwrap_or_default(),
        11 => entry.noun_singular_definite.clone().unwrap_or_default(),
        12 => entry.noun_plural_definite.clone().unwrap_or_default(),
        13 => entry.adjective_neuter_form.clone().unwrap_or_default(),
        14 => entry.adjective_plural_form.clone().unwrap_or_default(),
        15 => entry.adjective_comparative.clone().unwrap_or_default(),
        16 => entry
            .adjective_superlative_indefinite
            .clone()
            .unwrap_or_default(),
        17 => entry
            .adjective_superlative_definite
            .clone()
            .unwrap_or_default(),
        18 => entry.adverb_comparative.clone().unwrap_or_default(),
        19 => entry.adverb_superlative.clone().unwrap_or_default(),
        20 => entry.verb_present_participle.clone().unwrap_or_default(),
        21 => entry.verb_past_participle.clone().unwrap_or_default(),
        22 => entry.verb_passive_infinitive.clone().unwrap_or_default(),
        23 => entry.verb_passive_present.clone().unwrap_or_default(),
        24 => entry.verb_passive_past.clone().unwrap_or_default(),
        25 => entry
            .noun_singular_definite_genitive
            .clone()
            .unwrap_or_default(),
        26 => entry
            .noun_plural_definite_genitive
            .clone()
            .unwrap_or_default(),
        27 => entry
            .noun_singular_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        28 => entry
            .noun_plural_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        29 => entry.adjective_feminine_form.clone().unwrap_or_default(),
        30 => entry.pronoun_object.clone().unwrap_or_default(),
        31 => entry.pronoun_reflexive.clone().unwrap_or_default(),
        32 => entry.pronoun_plural_subject.clone().unwrap_or_default(),
        33 => entry.pronoun_plural_object.clone().unwrap_or_default(),
        34 => entry.pronoun_plural_reflexive.clone().unwrap_or_default(),
        35 => entry
            .determinative_feminine_form
            .clone()
            .unwrap_or_default(),
        36 => entry.determinative_neuter_form.clone().unwrap_or_default(),
        37 => entry.determinative_plural_form.clone().unwrap_or_default(),
        _ => String::new(),
    }
}
