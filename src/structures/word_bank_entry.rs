use serde::{Deserialize, Serialize};

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
    // 过去式：可为空
    pub past_tense: Option<String>,
    // 祈使式：可为空
    pub imperative: Option<String>,
    // 复数：可为空
    pub plural: Option<String>,
    // 单数特指：可为空
    pub singular_definite: Option<String>,
    // 复数特指：可为空
    pub plural_definite: Option<String>,
    // 对应中性：可为空
    pub neuter_form: Option<String>,
    // 对应复数：可为空
    pub plural_form: Option<String>,
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
