use std::cmp::Ordering;

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

use super::structures::{CsvWordEntry, PART_OF_SPEECH_OPTIONS, WordEntry};

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordEntry>, String> {
    let has_header = detect_word_bank_header(content);
    parse_word_bank_csv_inner(content, has_header)
}

pub fn serialize_word_bank_csv(entries: &[WordEntry]) -> Result<String, String> {
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
            "past_tense",
            "imperative",
            "plural",
            "singular_definite",
            "plural_definite",
            "neuter_form",
            "plural_form",
            "adjective_comparative",
            "adjective_superlative_indefinite",
            "adjective_superlative_definite",
            "adverb_comparative",
            "adverb_superlative",
        ])
        .map_err(|err| format!("CSV 写入失败: {err}"))?;

    for entry in entries {
        let normalized = word_entry_to_word_bank(entry)?;
        let csv_entry = CsvWordEntry::from_word_bank_entry(normalized);

        writer
            .write_record([
                csv_entry.id.as_str(),
                if csv_entry.selected { "true" } else { "false" },
                csv_entry.part_of_speech.as_str(),
                csv_entry.tags.as_str(),
                csv_entry.english.as_str(),
                csv_entry.chinese.as_str(),
                csv_entry.norwegian_base.as_str(),
                csv_entry.past_tense.as_str(),
                csv_entry.imperative.as_str(),
                csv_entry.plural.as_str(),
                csv_entry.singular_definite.as_str(),
                csv_entry.plural_definite.as_str(),
                csv_entry.neuter_form.as_str(),
                csv_entry.plural_form.as_str(),
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

pub(super) fn compare_entries_by_rules(a: &WordEntry, b: &WordEntry, rules: &[(usize, bool)]) -> Ordering {
    for (col_idx, asc) in rules {
        let ordering = compare_entries_by_column(a, b, *col_idx, *asc);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.id.cmp(&b.id)
}

pub(super) fn entry_matches_filter(entry: &WordEntry, query: &str, columns: &[bool]) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let query_lower = trimmed.to_lowercase();

    (0..19).any(|idx| {
        columns.get(idx).copied().unwrap_or(false)
            && column_value_text(entry, idx).to_lowercase().contains(&query_lower)
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
        7 => "past_tense",
        8 => "imperative",
        9 => "plural",
        10 => "singular_definite",
        11 => "plural_definite",
        12 => "neuter_form",
        13 => "plural_form",
        14 => "adjective_comparative",
        15 => "adjective_superlative_indefinite",
        16 => "adjective_superlative_definite",
        17 => "adverb_comparative",
        18 => "adverb_superlative",
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

fn parse_word_bank_csv_inner(content: &str, has_headers: bool) -> Result<Vec<WordEntry>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut entries = Vec::new();
    for row in reader.deserialize::<CsvWordEntry>() {
        let row = row.map_err(|err| format!("CSV 解析失败: {err}"))?;
        let normalized = row.to_word_bank_entry()?;
        entries.push(word_bank_to_word_entry(normalized));
    }

    Ok(entries)
}

fn compare_entries_by_column(a: &WordEntry, b: &WordEntry, col_idx: usize, ascending: bool) -> Ordering {
    let ordering = match col_idx {
        0 => a.id.cmp(&b.id),
        1 => a.selected.cmp(&b.selected),
        2 => a.part_of_speech.cmp(&b.part_of_speech),
        3 => a.tags.join("|").cmp(&b.tags.join("|")),
        4 => a.english.join("|").cmp(&b.english.join("|")),
        5 => a.chinese.join("|").cmp(&b.chinese.join("|")),
        6 => a.base_form.cmp(&b.base_form),
        7 => opt_cmp(&a.past_tense, &b.past_tense),
        8 => opt_cmp(&a.imperative, &b.imperative),
        9 => opt_cmp(&a.plural, &b.plural),
        10 => opt_cmp(&a.singular_definite, &b.singular_definite),
        11 => opt_cmp(&a.plural_definite, &b.plural_definite),
        12 => opt_cmp(&a.neuter_form, &b.neuter_form),
        13 => opt_cmp(&a.plural_form, &b.plural_form),
        14 => opt_cmp(&a.adjective_comparative, &b.adjective_comparative),
        15 => opt_cmp(
            &a.adjective_superlative_indefinite,
            &b.adjective_superlative_indefinite,
        ),
        16 => opt_cmp(&a.adjective_superlative_definite, &b.adjective_superlative_definite),
        17 => opt_cmp(&a.adverb_comparative, &b.adverb_comparative),
        18 => opt_cmp(&a.adverb_superlative, &b.adverb_superlative),
        _ => Ordering::Equal,
    };

    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

fn column_value_text(entry: &WordEntry, col_idx: usize) -> String {
    match col_idx {
        0 => entry.id.clone(),
        1 => {
            if entry.selected {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        2 => entry.part_of_speech.clone(),
        3 => entry.tags.join("|"),
        4 => entry.english.join("|"),
        5 => entry.chinese.join("|"),
        6 => entry.base_form.clone(),
        7 => entry.past_tense.clone().unwrap_or_default(),
        8 => entry.imperative.clone().unwrap_or_default(),
        9 => entry.plural.clone().unwrap_or_default(),
        10 => entry.singular_definite.clone().unwrap_or_default(),
        11 => entry.plural_definite.clone().unwrap_or_default(),
        12 => entry.neuter_form.clone().unwrap_or_default(),
        13 => entry.plural_form.clone().unwrap_or_default(),
        14 => entry.adjective_comparative.clone().unwrap_or_default(),
        15 => entry.adjective_superlative_indefinite.clone().unwrap_or_default(),
        16 => entry.adjective_superlative_definite.clone().unwrap_or_default(),
        17 => entry.adverb_comparative.clone().unwrap_or_default(),
        18 => entry.adverb_superlative.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn opt_cmp(a: &Option<String>, b: &Option<String>) -> Ordering {
    a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
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

fn parse_part_of_speech_key(raw: &str) -> Result<PartOfSpeech, String> {
    match raw.trim() {
        "verb" => Ok(PartOfSpeech::Verb),
        "noun" => Ok(PartOfSpeech::Noun),
        "adjective" => Ok(PartOfSpeech::Adjective),
        "adverb" => Ok(PartOfSpeech::Adverb),
        "cardinal_number" => Ok(PartOfSpeech::CardinalNumber),
        "ordinal_number" => Ok(PartOfSpeech::OrdinalNumber),
        "month" => Ok(PartOfSpeech::Month),
        "pronoun" => Ok(PartOfSpeech::Pronoun),
        "interrogative" => Ok(PartOfSpeech::Interrogative),
        _ => Err(format!(
            "词性不合法：{raw}，应为 {}",
            PART_OF_SPEECH_OPTIONS.join(", ")
        )),
    }
}

fn part_of_speech_to_key(pos: &PartOfSpeech) -> &'static str {
    match pos {
        PartOfSpeech::Verb => "verb",
        PartOfSpeech::Noun => "noun",
        PartOfSpeech::Adjective => "adjective",
        PartOfSpeech::Adverb => "adverb",
        PartOfSpeech::CardinalNumber => "cardinal_number",
        PartOfSpeech::OrdinalNumber => "ordinal_number",
        PartOfSpeech::Month => "month",
        PartOfSpeech::Pronoun => "pronoun",
        PartOfSpeech::Interrogative => "interrogative",
    }
}

fn word_bank_to_word_entry(entry: WordBankEntry) -> WordEntry {
    WordEntry {
        id: entry.id,
        selected: entry.selected,
        part_of_speech: part_of_speech_to_key(&entry.part_of_speech).to_string(),
        tags: entry.tags,
        english: entry.english,
        chinese: entry.chinese,
        base_form: entry.base_form,
        past_tense: entry.past_tense,
        imperative: entry.imperative,
        plural: entry.plural,
        singular_definite: entry.singular_definite,
        plural_definite: entry.plural_definite,
        neuter_form: entry.neuter_form,
        plural_form: entry.plural_form,
        adjective_comparative: entry.adjective_comparative,
        adjective_superlative_indefinite: entry.adjective_superlative_indefinite,
        adjective_superlative_definite: entry.adjective_superlative_definite,
        adverb_comparative: entry.adverb_comparative,
        adverb_superlative: entry.adverb_superlative,
    }
}

fn word_entry_to_word_bank(entry: &WordEntry) -> Result<WordBankEntry, String> {
    Ok(WordBankEntry {
        id: entry.id.clone(),
        selected: entry.selected,
        part_of_speech: parse_part_of_speech_key(&entry.part_of_speech)?,
        tags: entry.tags.clone(),
        english: entry.english.clone(),
        chinese: entry.chinese.clone(),
        base_form: entry.base_form.clone(),
        past_tense: entry.past_tense.clone(),
        imperative: entry.imperative.clone(),
        plural: entry.plural.clone(),
        singular_definite: entry.singular_definite.clone(),
        plural_definite: entry.plural_definite.clone(),
        neuter_form: entry.neuter_form.clone(),
        plural_form: entry.plural_form.clone(),
        adjective_comparative: entry.adjective_comparative.clone(),
        adjective_superlative_indefinite: entry.adjective_superlative_indefinite.clone(),
        adjective_superlative_definite: entry.adjective_superlative_definite.clone(),
        adverb_comparative: entry.adverb_comparative.clone(),
        adverb_superlative: entry.adverb_superlative.clone(),
    })
}

impl CsvWordEntry {
    fn to_word_bank_entry(self) -> Result<WordBankEntry, String> {
        Ok(WordBankEntry {
            id: self.id,
            selected: self.selected,
            part_of_speech: parse_part_of_speech_key(&self.part_of_speech)?,
            tags: parse_pipe_list(&self.tags),
            english: parse_pipe_list(&self.english),
            chinese: parse_pipe_list(&self.chinese),
            base_form: self.norwegian_base,
            past_tense: parse_optional_text(&self.past_tense),
            imperative: parse_optional_text(&self.imperative),
            plural: parse_optional_text(&self.plural),
            singular_definite: parse_optional_text(&self.singular_definite),
            plural_definite: parse_optional_text(&self.plural_definite),
            neuter_form: parse_optional_text(&self.neuter_form),
            plural_form: parse_optional_text(&self.plural_form),
            adjective_comparative: parse_optional_text(&self.adjective_comparative),
            adjective_superlative_indefinite: parse_optional_text(
                &self.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: parse_optional_text(
                &self.adjective_superlative_definite,
            ),
            adverb_comparative: parse_optional_text(&self.adverb_comparative),
            adverb_superlative: parse_optional_text(&self.adverb_superlative),
        })
    }

    fn from_word_bank_entry(entry: WordBankEntry) -> Self {
        Self {
            id: entry.id,
            selected: entry.selected,
            part_of_speech: part_of_speech_to_key(&entry.part_of_speech).to_string(),
            tags: entry.tags.join("|"),
            english: entry.english.join("|"),
            chinese: entry.chinese.join("|"),
            norwegian_base: entry.base_form,
            past_tense: optional_to_text(&entry.past_tense),
            imperative: optional_to_text(&entry.imperative),
            plural: optional_to_text(&entry.plural),
            singular_definite: optional_to_text(&entry.singular_definite),
            plural_definite: optional_to_text(&entry.plural_definite),
            neuter_form: optional_to_text(&entry.neuter_form),
            plural_form: optional_to_text(&entry.plural_form),
            adjective_comparative: optional_to_text(&entry.adjective_comparative),
            adjective_superlative_indefinite: optional_to_text(
                &entry.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: optional_to_text(&entry.adjective_superlative_definite),
            adverb_comparative: optional_to_text(&entry.adverb_comparative),
            adverb_superlative: optional_to_text(&entry.adverb_superlative),
        }
    }
}
