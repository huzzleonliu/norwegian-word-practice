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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize)]
struct CsvWordEntry {
    id: String,
    selected: bool,
    part_of_speech: String,
    tags: String,
    english: String,
    chinese: String,
    norwegian_base: String,
    past_tense: String,
    imperative: String,
    plural: String,
    singular_definite: String,
    plural_definite: String,
    neuter_form: String,
    plural_form: String,
    adjective_comparative: String,
    adjective_superlative_indefinite: String,
    adjective_superlative_definite: String,
    adverb_comparative: String,
    adverb_superlative: String,
}

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordEntry>, String> {
    let has_header = detect_word_bank_header(content);
    parse_word_bank_csv_inner(content, has_header)
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
        entries.push(WordEntry {
            id: row.id,
            selected: row.selected,
            part_of_speech: row.part_of_speech,
            tags: parse_pipe_list(&row.tags),
            english: parse_pipe_list(&row.english),
            chinese: parse_pipe_list(&row.chinese),
            base_form: row.norwegian_base,
            past_tense: parse_optional_text(&row.past_tense),
            imperative: parse_optional_text(&row.imperative),
            plural: parse_optional_text(&row.plural),
            singular_definite: parse_optional_text(&row.singular_definite),
            plural_definite: parse_optional_text(&row.plural_definite),
            neuter_form: parse_optional_text(&row.neuter_form),
            plural_form: parse_optional_text(&row.plural_form),
            adjective_comparative: parse_optional_text(&row.adjective_comparative),
            adjective_superlative_indefinite: parse_optional_text(
                &row.adjective_superlative_indefinite,
            ),
            adjective_superlative_definite: parse_optional_text(
                &row.adjective_superlative_definite,
            ),
            adverb_comparative: parse_optional_text(&row.adverb_comparative),
            adverb_superlative: parse_optional_text(&row.adverb_superlative),
        });
    }

    Ok(entries)
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
        let past_tense = optional_to_text(&entry.past_tense);
        let imperative = optional_to_text(&entry.imperative);
        let plural = optional_to_text(&entry.plural);
        let singular_definite = optional_to_text(&entry.singular_definite);
        let plural_definite = optional_to_text(&entry.plural_definite);
        let neuter_form = optional_to_text(&entry.neuter_form);
        let plural_form = optional_to_text(&entry.plural_form);
        let adjective_comparative = optional_to_text(&entry.adjective_comparative);
        let adjective_superlative_indefinite =
            optional_to_text(&entry.adjective_superlative_indefinite);
        let adjective_superlative_definite =
            optional_to_text(&entry.adjective_superlative_definite);
        let adverb_comparative = optional_to_text(&entry.adverb_comparative);
        let adverb_superlative = optional_to_text(&entry.adverb_superlative);

        writer
            .write_record([
                entry.id.as_str(),
                if entry.selected { "true" } else { "false" },
                entry.part_of_speech.as_str(),
                &entry.tags.join("|"),
                &entry.english.join("|"),
                &entry.chinese.join("|"),
                entry.base_form.as_str(),
                past_tense.as_str(),
                imperative.as_str(),
                plural.as_str(),
                singular_definite.as_str(),
                plural_definite.as_str(),
                neuter_form.as_str(),
                plural_form.as_str(),
                adjective_comparative.as_str(),
                adjective_superlative_indefinite.as_str(),
                adjective_superlative_definite.as_str(),
                adverb_comparative.as_str(),
                adverb_superlative.as_str(),
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
