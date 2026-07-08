use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WordEntry {
    pub id: String,
    pub part_of_speech: String,
    pub norwegian_base: String,
    pub chinese: Vec<String>,
    pub english: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CsvWordEntry {
    id: String,
    part_of_speech: String,
    norwegian_base: String,
    chinese: String,
    english: String,
    tags: String,
}

pub fn parse_word_bank_csv(content: &str) -> Result<Vec<WordEntry>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let mut entries = Vec::new();
    for row in reader.deserialize::<CsvWordEntry>() {
        let row = row.map_err(|err| format!("CSV 解析失败: {err}"))?;
        entries.push(WordEntry {
            id: row.id,
            part_of_speech: row.part_of_speech,
            norwegian_base: row.norwegian_base,
            chinese: parse_pipe_list(&row.chinese),
            english: parse_pipe_list(&row.english),
            tags: parse_pipe_list(&row.tags),
        });
    }

    Ok(entries)
}

pub fn serialize_word_bank_csv(entries: &[WordEntry]) -> Result<String, String> {
    let mut writer = csv::Writer::from_writer(Vec::<u8>::new());
    writer
        .write_record([
            "id",
            "part_of_speech",
            "norwegian_base",
            "chinese",
            "english",
            "tags",
        ])
        .map_err(|err| format!("CSV 写入失败: {err}"))?;

    for entry in entries {
        writer
            .write_record([
                entry.id.as_str(),
                entry.part_of_speech.as_str(),
                entry.norwegian_base.as_str(),
                &entry.chinese.join("|"),
                &entry.english.join("|"),
                &entry.tags.join("|"),
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
