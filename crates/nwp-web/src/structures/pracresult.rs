//! 练习结果模型：定义导入导出结构与每字段答题统计结构。

use serde::{Deserialize, Serialize};

/// 每个答案字段最多保留 5 条错法记录（用于展示最近/常见错误写法）。
pub const MAX_WRONG_ANSWERS_PER_FORM: usize = 5;

/// 练习结果（导入导出）：
/// - 用户名
/// - 加密密钥
/// - 选择的词库条目（id 列表）
/// - 已练习的词库条目统计表
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct PracticeResult {
    pub username: String,
    pub encryption_key: String,
    pub selected_word_entry_ids: Vec<String>,
    pub practiced_word_entries: Vec<PracticedWordEntryResult>,
}

/// 已练习的词库条目（表中的一行）。
///
/// 省略字段基于 `WordBankEntry` 的可练习字段补全：
/// english/chinese/base_form + 各词性变形字段。
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct PracticedWordEntryResult {
    /// 对应词库条目的序号（id）
    pub id: String,

    pub english: AnswerStats,
    pub chinese: AnswerStats,
    pub base_form: AnswerStats,

    pub verb_present_tense: AnswerStats,
    #[serde(alias = "past_tense")]
    pub verb_past_tense: AnswerStats,
    #[serde(alias = "imperative")]
    pub verb_imperative: AnswerStats,
    pub verb_present_participle: AnswerStats,
    pub verb_past_participle: AnswerStats,
    pub verb_passive_infinitive: AnswerStats,
    pub verb_passive_present: AnswerStats,
    pub verb_passive_past: AnswerStats,

    #[serde(alias = "plural")]
    pub noun_plural: AnswerStats,
    #[serde(alias = "singular_definite")]
    pub noun_singular_definite: AnswerStats,
    #[serde(alias = "plural_definite")]
    pub noun_plural_definite: AnswerStats,
    pub noun_singular_definite_genitive: AnswerStats,
    pub noun_plural_definite_genitive: AnswerStats,
    pub noun_singular_indefinite_genitive: AnswerStats,
    pub noun_plural_indefinite_genitive: AnswerStats,

    pub adjective_feminine_form: AnswerStats,
    #[serde(alias = "neuter_form")]
    pub adjective_neuter_form: AnswerStats,
    #[serde(alias = "plural_form")]
    pub adjective_plural_form: AnswerStats,
    pub adjective_comparative: AnswerStats,
    pub adjective_superlative_indefinite: AnswerStats,
    pub adjective_superlative_definite: AnswerStats,

    pub pronoun_object: AnswerStats,
    pub pronoun_reflexive: AnswerStats,
    pub pronoun_plural_subject: AnswerStats,
    pub pronoun_plural_object: AnswerStats,
    pub pronoun_plural_reflexive: AnswerStats,

    pub determinative_feminine_form: AnswerStats,
    pub determinative_neuter_form: AnswerStats,
    pub determinative_plural_form: AnswerStats,

    pub adverb_comparative: AnswerStats,
    pub adverb_superlative: AnswerStats,
}

/// 单个答案字段的练习统计：
/// - 正确次数
/// - 错误次数
/// - 错法记录（建议最大长度：`MAX_WRONG_ANSWERS_PER_FORM`）
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct AnswerStats {
    pub correct_count: usize,
    pub wrong_count: usize,
    pub wrong_answers: Vec<String>,
}
