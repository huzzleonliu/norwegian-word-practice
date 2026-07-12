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
pub enum PartOfSpeech {
    Verb,
    Noun,
    Adjective,
    Adverb,
    CardinalNumber,
    OrdinalNumber,
    Month,
    Pronoun,
    Interrogative,
}

impl PartOfSpeech {
    pub fn as_key(&self) -> &'static str {
        match self {
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

    pub fn from_key(raw: &str) -> Result<Self, String> {
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
                "词性不合法，请从预设选项中选择：{}",
                PART_OF_SPEECH_OPTIONS.join(", ")
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WordBankEntry {
    // 序号：唯一，不可为空；建议与原型关联
    pub id: String,
    // 是否选中：默认 true
    pub selected: bool,
    // 词性：从固定枚举中选 1
    pub part_of_speech: PartOfSpeech,
    // tag：可为空
    pub tags: Vec<String>,
    // 对应英文：可为空
    pub english: Vec<String>,
    // 对应中文：不可为空
    pub chinese: Vec<String>,
    // 原型：不可为空
    pub base_form: String,
    // 现在时：可为空（动词时必填）
    pub verb_present_tense: Option<String>,
    // 过去式：可为空
    pub verb_past_tense: Option<String>,
    // 祈使式：可为空
    pub verb_imperative: Option<String>,
    // 复数：可为空
    pub noun_plural: Option<String>,
    // 单数特指：可为空
    pub noun_singular_definite: Option<String>,
    // 复数特指：可为空
    pub noun_plural_definite: Option<String>,
    // 对应中性：可为空
    pub adjective_neuter_form: Option<String>,
    // 对应复数：可为空
    pub adjective_plural_form: Option<String>,
    // 形容词比较级：可为空
    pub adjective_comparative: Option<String>,
    // 形容词最高级泛指：可为空
    pub adjective_superlative_indefinite: Option<String>,
    // 形容词最高级特指：可为空
    pub adjective_superlative_definite: Option<String>,
    // 副词比较级：可为空
    pub adverb_comparative: Option<String>,
    // 副词最高级：可为空
    pub adverb_superlative: Option<String>,
}
