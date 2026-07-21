//! 词库领域模型：定义词性枚举、语言枚举与核心词条结构 `WordBankEntry`。

use serde::{Deserialize, Serialize};

pub const PART_OF_SPEECH_OPTIONS: [&str; 11] = [
    "verb",
    "noun",
    "adjective",
    "pronoun",
    "determinative",
    "adverb",
    "preposition",
    "conjunction",
    "subjunction",
    "interjection",
    "phrase",
];

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UiLanguage {
    Zh,
    En,
}

impl UiLanguage {
    /// 在中文/英文界面语言之间切换。
    pub fn toggle(self) -> Self {
        match self {
            UiLanguage::Zh => UiLanguage::En,
            UiLanguage::En => UiLanguage::Zh,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum UiTheme {
    Dark,
    Light,
}

impl UiTheme {
    /// 在黑夜/白天主题之间切换。
    pub fn toggle(self) -> Self {
        match self {
            UiTheme::Dark => UiTheme::Light,
            UiTheme::Light => UiTheme::Dark,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PartOfSpeech {
    Verb,
    Noun,
    Adjective,
    Pronoun,
    Determinative,
    Adverb,
    Preposition,
    Conjunction,
    Subjunction,
    Interjection,
    /// 词组：除中文、id、原型、selected、词性外，其余字段均可为空。
    Phrase,
}

impl PartOfSpeech {
    /// 枚举到稳定 key（用于存储、筛选、哈希与 CSV）。
    pub fn as_key(&self) -> &'static str {
        match self {
            PartOfSpeech::Verb => "verb",
            PartOfSpeech::Noun => "noun",
            PartOfSpeech::Adjective => "adjective",
            PartOfSpeech::Pronoun => "pronoun",
            PartOfSpeech::Determinative => "determinative",
            PartOfSpeech::Adverb => "adverb",
            PartOfSpeech::Preposition => "preposition",
            PartOfSpeech::Conjunction => "conjunction",
            PartOfSpeech::Subjunction => "subjunction",
            PartOfSpeech::Interjection => "interjection",
            PartOfSpeech::Phrase => "phrase",
        }
    }

    pub fn from_key(raw: &str) -> Result<Self, String> {
        match raw.trim() {
            "verb" => Ok(PartOfSpeech::Verb),
            "noun" => Ok(PartOfSpeech::Noun),
            "adjective" => Ok(PartOfSpeech::Adjective),
            "pronoun" => Ok(PartOfSpeech::Pronoun),
            "determinative" => Ok(PartOfSpeech::Determinative),
            "adverb" => Ok(PartOfSpeech::Adverb),
            "preposition" => Ok(PartOfSpeech::Preposition),
            "conjunction" => Ok(PartOfSpeech::Conjunction),
            "subjunction" => Ok(PartOfSpeech::Subjunction),
            "interjection" => Ok(PartOfSpeech::Interjection),
            "phrase" => Ok(PartOfSpeech::Phrase),
            _ => Err(format!(
                "词性不合法，请从预设选项中选择：{}",
                PART_OF_SPEECH_OPTIONS.join(", ")
            )),
        }
    }

    pub fn display_name(&self, language: UiLanguage) -> &'static str {
        match language {
            UiLanguage::Zh => match self {
                PartOfSpeech::Verb => "动词",
                PartOfSpeech::Noun => "名词",
                PartOfSpeech::Adjective => "形容词",
                PartOfSpeech::Pronoun => "代词",
                PartOfSpeech::Determinative => "限定词",
                PartOfSpeech::Adverb => "副词",
                PartOfSpeech::Preposition => "介词",
                PartOfSpeech::Conjunction => "并列连词",
                PartOfSpeech::Subjunction => "从属连词",
                PartOfSpeech::Interjection => "感叹词",
                PartOfSpeech::Phrase => "词组",
            },
            UiLanguage::En => match self {
                PartOfSpeech::Verb => "Verb",
                PartOfSpeech::Noun => "Noun",
                PartOfSpeech::Adjective => "Adjective",
                PartOfSpeech::Pronoun => "Pronoun",
                PartOfSpeech::Determinative => "Determinative",
                PartOfSpeech::Adverb => "Adverb",
                PartOfSpeech::Preposition => "Preposition",
                PartOfSpeech::Conjunction => "Conjunction",
                PartOfSpeech::Subjunction => "Subjunction",
                PartOfSpeech::Interjection => "Interjection",
                PartOfSpeech::Phrase => "Phrase",
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WordBankEntry {
    // 序号：唯一，不可为空；由词性+词形字段哈希生成
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
    // 现在分词：可为空
    pub verb_present_participle: Option<String>,
    // 过去分词：可为空
    pub verb_past_participle: Option<String>,
    // 被动不定式：可为空
    pub verb_passive_infinitive: Option<String>,
    // 被动现在时：可为空
    pub verb_passive_present: Option<String>,
    // 被动过去时：可为空（较少见）
    pub verb_passive_past: Option<String>,
    // 复数：可为空
    pub noun_plural: Option<String>,
    // 单数特指：可为空
    pub noun_singular_definite: Option<String>,
    // 复数特指：可为空
    pub noun_plural_definite: Option<String>,
    // 单数特指所有格：可为空
    pub noun_singular_definite_genitive: Option<String>,
    // 复数特指所有格：可为空
    pub noun_plural_definite_genitive: Option<String>,
    // 单数泛指所有格：可为空（极少见）
    pub noun_singular_indefinite_genitive: Option<String>,
    // 复数泛指所有格：可为空（极少见）
    pub noun_plural_indefinite_genitive: Option<String>,
    // 对应阴性：可为空
    pub adjective_feminine_form: Option<String>,
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
    // 代词宾格：可为空
    pub pronoun_object: Option<String>,
    // 代词反身：可为空
    pub pronoun_reflexive: Option<String>,
    // 代词复数主格：可为空
    pub pronoun_plural_subject: Option<String>,
    // 代词复数宾格：可为空
    pub pronoun_plural_object: Option<String>,
    // 代词复数反身：可为空
    pub pronoun_plural_reflexive: Option<String>,
    // 限定词阴性：可为空
    pub determinative_feminine_form: Option<String>,
    // 限定词中性：可为空
    pub determinative_neuter_form: Option<String>,
    // 限定词复数：可为空
    pub determinative_plural_form: Option<String>,
    // 副词比较级：可为空
    pub adverb_comparative: Option<String>,
    // 副词最高级：可为空
    pub adverb_superlative: Option<String>,
}
