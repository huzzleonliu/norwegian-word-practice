use serde::{Deserialize, Serialize};

use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

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
        let normalized = row.to_word_bank_entry()?;
        entries.push(word_bank_to_word_entry(normalized));
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
