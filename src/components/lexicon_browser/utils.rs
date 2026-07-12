use std::cmp::Ordering;

use crate::structures::word_bank_entry::WordBankEntry;

use super::structures::CsvWordEntry;

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordBankEntry>, String> {
    let has_header = detect_word_bank_header(content);
    parse_word_bank_csv_inner(content, has_header)
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

    (0..20).any(|idx| {
        columns.get(idx).copied().unwrap_or(false)
            && column_value_text(entry, idx)
                .to_lowercase()
                .contains(&query_lower)
    })
}

pub(super) fn header_name(col_idx: usize) -> &'static str {
    match col_idx {
        0 => "id",
        1 => "selected",
        2 => "part_of_speech",
        3 => "tags",
        4 => "english",
        5 => "chinese",
        6 => "base_form",
        7 => "verb_present_tense",
        8 => "verb_past_tense",
        9 => "verb_imperative",
        10 => "noun_plural",
        11 => "noun_singular_definite",
        12 => "noun_plural_definite",
        13 => "adjective_neuter_form",
        14 => "adjective_plural_form",
        15 => "adjective_comparative",
        16 => "adjective_superlative_indefinite",
        17 => "adjective_superlative_definite",
        18 => "adverb_comparative",
        19 => "adverb_superlative",
        _ => "unknown",
    }
}

pub(super) fn format_sort_rules(rules: &[(usize, bool)]) -> String {
    if rules.is_empty() {
        return "未应用排序".to_string();
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
        .join("，")
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
    let normalized = first_non_empty_line.trim().to_ascii_lowercase();
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
    for row in reader.deserialize::<CsvWordEntry>() {
        let row = row.map_err(|err| format!("CSV 解析失败: {err}"))?;
        entries.push(WordBankEntry::try_from(row)?);
    }

    Ok(entries)
}

fn compare_entries_by_column(
    a: &WordBankEntry,
    b: &WordBankEntry,
    col_idx: usize,
    ascending: bool,
) -> Ordering {
    let ordering = match col_idx {
        0 => a.id.cmp(&b.id),
        1 => a.selected.cmp(&b.selected),
        2 => a.part_of_speech.as_key().cmp(b.part_of_speech.as_key()),
        3 => a.tags.join("|").cmp(&b.tags.join("|")),
        4 => a.english.join("|").cmp(&b.english.join("|")),
        5 => a.chinese.join("|").cmp(&b.chinese.join("|")),
        6 => a.base_form.cmp(&b.base_form),
        7 => opt_cmp(&a.verb_present_tense, &b.verb_present_tense),
        8 => opt_cmp(&a.verb_past_tense, &b.verb_past_tense),
        9 => opt_cmp(&a.verb_imperative, &b.verb_imperative),
        10 => opt_cmp(&a.noun_plural, &b.noun_plural),
        11 => opt_cmp(&a.noun_singular_definite, &b.noun_singular_definite),
        12 => opt_cmp(&a.noun_plural_definite, &b.noun_plural_definite),
        13 => opt_cmp(&a.adjective_neuter_form, &b.adjective_neuter_form),
        14 => opt_cmp(&a.adjective_plural_form, &b.adjective_plural_form),
        15 => opt_cmp(&a.adjective_comparative, &b.adjective_comparative),
        16 => opt_cmp(
            &a.adjective_superlative_indefinite,
            &b.adjective_superlative_indefinite,
        ),
        17 => opt_cmp(
            &a.adjective_superlative_definite,
            &b.adjective_superlative_definite,
        ),
        18 => opt_cmp(&a.adverb_comparative, &b.adverb_comparative),
        19 => opt_cmp(&a.adverb_superlative, &b.adverb_superlative),
        _ => Ordering::Equal,
    };

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
        _ => String::new(),
    }
}

fn opt_cmp(a: &Option<String>, b: &Option<String>) -> Ordering {
    a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
}
