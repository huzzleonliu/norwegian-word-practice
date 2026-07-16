//! 字段元数据单一真相源：
//! - 维护字段 key、分组、是否参与哈希、练习可用性、默认可见性
//! - 统一字段标签、字段取值与练习统计字段映射

use crate::structures::pracresult::{AnswerStats, PracticedWordEntryResult};
use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};

pub const NONE_FIELD_KEY: &str = "none";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldMeta {
    pub key: &'static str,
    pub group: &'static str,
    pub hash_relevant: bool,
    pub practice_prompt: bool,
    pub practice_answer: bool,
    pub default_search_visible: bool,
    pub default_answer_selected: bool,
    pub zh_label: &'static str,
    pub en_label: &'static str,
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

pub const PROMPT_FIELD_OPTIONS: [&str; 37] = [
    NONE_FIELD_KEY,
    "part_of_speech",
    "tags",
    "english",
    "chinese",
    "base_form",
    "verb_present_tense",
    "verb_past_tense",
    "verb_imperative",
    "verb_present_participle",
    "verb_past_participle",
    "verb_passive_infinitive",
    "verb_passive_present",
    "verb_passive_past",
    "noun_plural",
    "noun_singular_definite",
    "noun_plural_definite",
    "noun_singular_definite_genitive",
    "noun_plural_definite_genitive",
    "noun_singular_indefinite_genitive",
    "noun_plural_indefinite_genitive",
    "adjective_feminine_form",
    "adjective_neuter_form",
    "adjective_plural_form",
    "adjective_comparative",
    "adjective_superlative_indefinite",
    "adjective_superlative_definite",
    "pronoun_object",
    "pronoun_reflexive",
    "pronoun_plural_subject",
    "pronoun_plural_object",
    "pronoun_plural_reflexive",
    "determinative_feminine_form",
    "determinative_neuter_form",
    "determinative_plural_form",
    "adverb_comparative",
    "adverb_superlative",
];

pub const ANSWER_FIELD_OPTIONS: [&str; 34] = [
    "english",
    "chinese",
    "base_form",
    "verb_present_tense",
    "verb_past_tense",
    "verb_imperative",
    "verb_present_participle",
    "verb_past_participle",
    "verb_passive_infinitive",
    "verb_passive_present",
    "verb_passive_past",
    "noun_plural",
    "noun_singular_definite",
    "noun_plural_definite",
    "noun_singular_definite_genitive",
    "noun_plural_definite_genitive",
    "noun_singular_indefinite_genitive",
    "noun_plural_indefinite_genitive",
    "adjective_feminine_form",
    "adjective_neuter_form",
    "adjective_plural_form",
    "adjective_comparative",
    "adjective_superlative_indefinite",
    "adjective_superlative_definite",
    "pronoun_object",
    "pronoun_reflexive",
    "pronoun_plural_subject",
    "pronoun_plural_object",
    "pronoun_plural_reflexive",
    "determinative_feminine_form",
    "determinative_neuter_form",
    "determinative_plural_form",
    "adverb_comparative",
    "adverb_superlative",
];

pub const ANSWER_FIELD_GROUPS: [(&str, &[&str]); 7] = [
    ("core", &["english", "chinese", "base_form"]),
    (
        "verb",
        &[
            "verb_present_tense",
            "verb_past_tense",
            "verb_imperative",
            "verb_present_participle",
            "verb_past_participle",
            "verb_passive_infinitive",
            "verb_passive_present",
            "verb_passive_past",
        ],
    ),
    (
        "noun",
        &[
            "noun_plural",
            "noun_singular_definite",
            "noun_plural_definite",
            "noun_singular_definite_genitive",
            "noun_plural_definite_genitive",
            "noun_singular_indefinite_genitive",
            "noun_plural_indefinite_genitive",
        ],
    ),
    (
        "adjective",
        &[
            "adjective_feminine_form",
            "adjective_neuter_form",
            "adjective_plural_form",
            "adjective_comparative",
            "adjective_superlative_indefinite",
            "adjective_superlative_definite",
        ],
    ),
    (
        "pronoun",
        &[
            "pronoun_object",
            "pronoun_reflexive",
            "pronoun_plural_subject",
            "pronoun_plural_object",
            "pronoun_plural_reflexive",
        ],
    ),
    (
        "determinative",
        &[
            "determinative_feminine_form",
            "determinative_neuter_form",
            "determinative_plural_form",
        ],
    ),
    ("adverb", &["adverb_comparative", "adverb_superlative"]),
];

pub const FIELD_META: [FieldMeta; 38] = [
    FieldMeta {
        key: "id",
        group: "core",
        hash_relevant: false,
        practice_prompt: false,
        practice_answer: false,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "序号",
        en_label: "ID",
    },
    FieldMeta {
        key: "selected",
        group: "core",
        hash_relevant: false,
        practice_prompt: false,
        practice_answer: false,
        default_search_visible: true,
        default_answer_selected: false,
        zh_label: "选中",
        en_label: "Selected",
    },
    FieldMeta {
        key: "part_of_speech",
        group: "core",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: false,
        default_search_visible: true,
        default_answer_selected: false,
        zh_label: "词性",
        en_label: "Part of Speech",
    },
    FieldMeta {
        key: "tags",
        group: "core",
        hash_relevant: false,
        practice_prompt: true,
        practice_answer: false,
        default_search_visible: true,
        default_answer_selected: false,
        zh_label: "标签",
        en_label: "Tags",
    },
    FieldMeta {
        key: "english",
        group: "core",
        hash_relevant: false,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: false,
        zh_label: "英文",
        en_label: "English",
    },
    FieldMeta {
        key: "chinese",
        group: "core",
        hash_relevant: false,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: false,
        zh_label: "中文",
        en_label: "Chinese",
    },
    FieldMeta {
        key: "base_form",
        group: "core",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "原型",
        en_label: "Base Form",
    },
    FieldMeta {
        key: "verb_present_tense",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_现在时",
        en_label: "Verb Present",
    },
    FieldMeta {
        key: "verb_past_tense",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_过去式",
        en_label: "Verb Past",
    },
    FieldMeta {
        key: "verb_imperative",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_祈使式",
        en_label: "Verb Imperative",
    },
    FieldMeta {
        key: "noun_plural",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "名词_复数",
        en_label: "Noun Plural",
    },
    FieldMeta {
        key: "noun_singular_definite",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "名词_单数特指",
        en_label: "Noun Singular Definite",
    },
    FieldMeta {
        key: "noun_plural_definite",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "名词_复数特指",
        en_label: "Noun Plural Definite",
    },
    FieldMeta {
        key: "adjective_neuter_form",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_中性",
        en_label: "Adjective Neuter",
    },
    FieldMeta {
        key: "adjective_plural_form",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_复数",
        en_label: "Adjective Plural",
    },
    FieldMeta {
        key: "adjective_comparative",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_比较级",
        en_label: "Adjective Comparative",
    },
    FieldMeta {
        key: "adjective_superlative_indefinite",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_最高级泛指",
        en_label: "Adjective Superlative Indefinite",
    },
    FieldMeta {
        key: "adjective_superlative_definite",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_最高级特指",
        en_label: "Adjective Superlative Definite",
    },
    FieldMeta {
        key: "adverb_comparative",
        group: "adverb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "副词_比较级",
        en_label: "Adverb Comparative",
    },
    FieldMeta {
        key: "adverb_superlative",
        group: "adverb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "副词_最高级",
        en_label: "Adverb Superlative",
    },
    FieldMeta {
        key: "verb_present_participle",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_现在分词",
        en_label: "Verb Present Participle",
    },
    FieldMeta {
        key: "verb_past_participle",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_过去分词",
        en_label: "Verb Past Participle",
    },
    FieldMeta {
        key: "verb_passive_infinitive",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_被动不定式",
        en_label: "Verb Passive Infinitive",
    },
    FieldMeta {
        key: "verb_passive_present",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "动词_被动现在时",
        en_label: "Verb Passive Present",
    },
    FieldMeta {
        key: "verb_passive_past",
        group: "verb",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "动词_被动过去时",
        en_label: "Verb Passive Past",
    },
    FieldMeta {
        key: "noun_singular_definite_genitive",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "名词_单数特指所有格",
        en_label: "Noun Singular Definite Genitive",
    },
    FieldMeta {
        key: "noun_plural_definite_genitive",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "名词_复数特指所有格",
        en_label: "Noun Plural Definite Genitive",
    },
    FieldMeta {
        key: "noun_singular_indefinite_genitive",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "名词_单数泛指所有格",
        en_label: "Noun Singular Indefinite Genitive",
    },
    FieldMeta {
        key: "noun_plural_indefinite_genitive",
        group: "noun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: false,
        default_answer_selected: false,
        zh_label: "名词_复数泛指所有格",
        en_label: "Noun Plural Indefinite Genitive",
    },
    FieldMeta {
        key: "adjective_feminine_form",
        group: "adjective",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "形容词_阴性",
        en_label: "Adjective Feminine",
    },
    FieldMeta {
        key: "pronoun_object",
        group: "pronoun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "代词_宾格",
        en_label: "Pronoun Object",
    },
    FieldMeta {
        key: "pronoun_reflexive",
        group: "pronoun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "代词_反身",
        en_label: "Pronoun Reflexive",
    },
    FieldMeta {
        key: "pronoun_plural_subject",
        group: "pronoun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "代词_复数主格",
        en_label: "Pronoun Plural Subject",
    },
    FieldMeta {
        key: "pronoun_plural_object",
        group: "pronoun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "代词_复数宾格",
        en_label: "Pronoun Plural Object",
    },
    FieldMeta {
        key: "pronoun_plural_reflexive",
        group: "pronoun",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "代词_复数反身",
        en_label: "Pronoun Plural Reflexive",
    },
    FieldMeta {
        key: "determinative_feminine_form",
        group: "determinative",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "限定词_阴性",
        en_label: "Determinative Feminine",
    },
    FieldMeta {
        key: "determinative_neuter_form",
        group: "determinative",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "限定词_中性",
        en_label: "Determinative Neuter",
    },
    FieldMeta {
        key: "determinative_plural_form",
        group: "determinative",
        hash_relevant: true,
        practice_prompt: true,
        practice_answer: true,
        default_search_visible: true,
        default_answer_selected: true,
        zh_label: "限定词_复数",
        en_label: "Determinative Plural",
    },
];

pub fn field_meta(key: &str) -> Option<&'static FieldMeta> {
    FIELD_META.iter().find(|meta| meta.key == key)
}

pub fn field_label(lang: UiLanguage, key: &str) -> &'static str {
    if key == NONE_FIELD_KEY {
        return match lang {
            UiLanguage::Zh => "无",
            UiLanguage::En => "None",
        };
    }
    match (lang, field_meta(key)) {
        (UiLanguage::Zh, Some(meta)) => meta.zh_label,
        (UiLanguage::En, Some(meta)) => meta.en_label,
        _ => "unknown",
    }
}

pub fn default_search_column_visibility() -> Vec<bool> {
    DATA_COLUMN_KEYS
        .iter()
        .map(|key| {
            field_meta(key)
                .map(|meta| meta.default_search_visible)
                .unwrap_or(true)
        })
        .collect()
}

pub fn default_answer_fields() -> Vec<String> {
    ANSWER_FIELD_OPTIONS
        .iter()
        .filter_map(|key| {
            field_meta(key)
                .filter(|meta| meta.default_answer_selected)
                .map(|meta| meta.key.to_string())
        })
        .collect()
}

pub fn entry_field_value(entry: &WordBankEntry, field: &str) -> String {
    match field {
        "id" => entry.id.clone(),
        "selected" => {
            if entry.selected {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        "part_of_speech" => entry.part_of_speech.as_key().to_string(),
        "tags" => entry.tags.join(" | "),
        "english" => entry.english.join(" | "),
        "chinese" => entry.chinese.join(" | "),
        "base_form" => entry.base_form.clone(),
        "verb_present_tense" => entry.verb_present_tense.clone().unwrap_or_default(),
        "verb_past_tense" => entry.verb_past_tense.clone().unwrap_or_default(),
        "verb_imperative" => entry.verb_imperative.clone().unwrap_or_default(),
        "verb_present_participle" => entry.verb_present_participle.clone().unwrap_or_default(),
        "verb_past_participle" => entry.verb_past_participle.clone().unwrap_or_default(),
        "verb_passive_infinitive" => entry.verb_passive_infinitive.clone().unwrap_or_default(),
        "verb_passive_present" => entry.verb_passive_present.clone().unwrap_or_default(),
        "verb_passive_past" => entry.verb_passive_past.clone().unwrap_or_default(),
        "noun_plural" => entry.noun_plural.clone().unwrap_or_default(),
        "noun_singular_definite" => entry.noun_singular_definite.clone().unwrap_or_default(),
        "noun_plural_definite" => entry.noun_plural_definite.clone().unwrap_or_default(),
        "noun_singular_definite_genitive" => entry
            .noun_singular_definite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_plural_definite_genitive" => entry
            .noun_plural_definite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_singular_indefinite_genitive" => entry
            .noun_singular_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_plural_indefinite_genitive" => entry
            .noun_plural_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        "adjective_feminine_form" => entry.adjective_feminine_form.clone().unwrap_or_default(),
        "adjective_neuter_form" => entry.adjective_neuter_form.clone().unwrap_or_default(),
        "adjective_plural_form" => entry.adjective_plural_form.clone().unwrap_or_default(),
        "adjective_comparative" => entry.adjective_comparative.clone().unwrap_or_default(),
        "adjective_superlative_indefinite" => entry
            .adjective_superlative_indefinite
            .clone()
            .unwrap_or_default(),
        "adjective_superlative_definite" => entry
            .adjective_superlative_definite
            .clone()
            .unwrap_or_default(),
        "pronoun_object" => entry.pronoun_object.clone().unwrap_or_default(),
        "pronoun_reflexive" => entry.pronoun_reflexive.clone().unwrap_or_default(),
        "pronoun_plural_subject" => entry.pronoun_plural_subject.clone().unwrap_or_default(),
        "pronoun_plural_object" => entry.pronoun_plural_object.clone().unwrap_or_default(),
        "pronoun_plural_reflexive" => entry.pronoun_plural_reflexive.clone().unwrap_or_default(),
        "determinative_feminine_form" => entry
            .determinative_feminine_form
            .clone()
            .unwrap_or_default(),
        "determinative_neuter_form" => entry.determinative_neuter_form.clone().unwrap_or_default(),
        "determinative_plural_form" => entry.determinative_plural_form.clone().unwrap_or_default(),
        "adverb_comparative" => entry.adverb_comparative.clone().unwrap_or_default(),
        "adverb_superlative" => entry.adverb_superlative.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn answer_stats_ref<'a>(
    practiced_entry: &'a PracticedWordEntryResult,
    field: &str,
) -> Option<&'a AnswerStats> {
    match field {
        "english" => Some(&practiced_entry.english),
        "chinese" => Some(&practiced_entry.chinese),
        "base_form" => Some(&practiced_entry.base_form),
        "verb_present_tense" => Some(&practiced_entry.verb_present_tense),
        "verb_past_tense" => Some(&practiced_entry.verb_past_tense),
        "verb_imperative" => Some(&practiced_entry.verb_imperative),
        "verb_present_participle" => Some(&practiced_entry.verb_present_participle),
        "verb_past_participle" => Some(&practiced_entry.verb_past_participle),
        "verb_passive_infinitive" => Some(&practiced_entry.verb_passive_infinitive),
        "verb_passive_present" => Some(&practiced_entry.verb_passive_present),
        "verb_passive_past" => Some(&practiced_entry.verb_passive_past),
        "noun_plural" => Some(&practiced_entry.noun_plural),
        "noun_singular_definite" => Some(&practiced_entry.noun_singular_definite),
        "noun_plural_definite" => Some(&practiced_entry.noun_plural_definite),
        "noun_singular_definite_genitive" => Some(&practiced_entry.noun_singular_definite_genitive),
        "noun_plural_definite_genitive" => Some(&practiced_entry.noun_plural_definite_genitive),
        "noun_singular_indefinite_genitive" => {
            Some(&practiced_entry.noun_singular_indefinite_genitive)
        }
        "noun_plural_indefinite_genitive" => Some(&practiced_entry.noun_plural_indefinite_genitive),
        "adjective_feminine_form" => Some(&practiced_entry.adjective_feminine_form),
        "adjective_neuter_form" => Some(&practiced_entry.adjective_neuter_form),
        "adjective_plural_form" => Some(&practiced_entry.adjective_plural_form),
        "adjective_comparative" => Some(&practiced_entry.adjective_comparative),
        "adjective_superlative_indefinite" => {
            Some(&practiced_entry.adjective_superlative_indefinite)
        }
        "adjective_superlative_definite" => Some(&practiced_entry.adjective_superlative_definite),
        "pronoun_object" => Some(&practiced_entry.pronoun_object),
        "pronoun_reflexive" => Some(&practiced_entry.pronoun_reflexive),
        "pronoun_plural_subject" => Some(&practiced_entry.pronoun_plural_subject),
        "pronoun_plural_object" => Some(&practiced_entry.pronoun_plural_object),
        "pronoun_plural_reflexive" => Some(&practiced_entry.pronoun_plural_reflexive),
        "determinative_feminine_form" => Some(&practiced_entry.determinative_feminine_form),
        "determinative_neuter_form" => Some(&practiced_entry.determinative_neuter_form),
        "determinative_plural_form" => Some(&practiced_entry.determinative_plural_form),
        "adverb_comparative" => Some(&practiced_entry.adverb_comparative),
        "adverb_superlative" => Some(&practiced_entry.adverb_superlative),
        _ => None,
    }
}

pub fn answer_stats_mut<'a>(
    practiced_entry: &'a mut PracticedWordEntryResult,
    field: &str,
) -> Option<&'a mut AnswerStats> {
    match field {
        "english" => Some(&mut practiced_entry.english),
        "chinese" => Some(&mut practiced_entry.chinese),
        "base_form" => Some(&mut practiced_entry.base_form),
        "verb_present_tense" => Some(&mut practiced_entry.verb_present_tense),
        "verb_past_tense" => Some(&mut practiced_entry.verb_past_tense),
        "verb_imperative" => Some(&mut practiced_entry.verb_imperative),
        "verb_present_participle" => Some(&mut practiced_entry.verb_present_participle),
        "verb_past_participle" => Some(&mut practiced_entry.verb_past_participle),
        "verb_passive_infinitive" => Some(&mut practiced_entry.verb_passive_infinitive),
        "verb_passive_present" => Some(&mut practiced_entry.verb_passive_present),
        "verb_passive_past" => Some(&mut practiced_entry.verb_passive_past),
        "noun_plural" => Some(&mut practiced_entry.noun_plural),
        "noun_singular_definite" => Some(&mut practiced_entry.noun_singular_definite),
        "noun_plural_definite" => Some(&mut practiced_entry.noun_plural_definite),
        "noun_singular_definite_genitive" => {
            Some(&mut practiced_entry.noun_singular_definite_genitive)
        }
        "noun_plural_definite_genitive" => Some(&mut practiced_entry.noun_plural_definite_genitive),
        "noun_singular_indefinite_genitive" => {
            Some(&mut practiced_entry.noun_singular_indefinite_genitive)
        }
        "noun_plural_indefinite_genitive" => {
            Some(&mut practiced_entry.noun_plural_indefinite_genitive)
        }
        "adjective_feminine_form" => Some(&mut practiced_entry.adjective_feminine_form),
        "adjective_neuter_form" => Some(&mut practiced_entry.adjective_neuter_form),
        "adjective_plural_form" => Some(&mut practiced_entry.adjective_plural_form),
        "adjective_comparative" => Some(&mut practiced_entry.adjective_comparative),
        "adjective_superlative_indefinite" => {
            Some(&mut practiced_entry.adjective_superlative_indefinite)
        }
        "adjective_superlative_definite" => {
            Some(&mut practiced_entry.adjective_superlative_definite)
        }
        "pronoun_object" => Some(&mut practiced_entry.pronoun_object),
        "pronoun_reflexive" => Some(&mut practiced_entry.pronoun_reflexive),
        "pronoun_plural_subject" => Some(&mut practiced_entry.pronoun_plural_subject),
        "pronoun_plural_object" => Some(&mut practiced_entry.pronoun_plural_object),
        "pronoun_plural_reflexive" => Some(&mut practiced_entry.pronoun_plural_reflexive),
        "determinative_feminine_form" => Some(&mut practiced_entry.determinative_feminine_form),
        "determinative_neuter_form" => Some(&mut practiced_entry.determinative_neuter_form),
        "determinative_plural_form" => Some(&mut practiced_entry.determinative_plural_form),
        "adverb_comparative" => Some(&mut practiced_entry.adverb_comparative),
        "adverb_superlative" => Some(&mut practiced_entry.adverb_superlative),
        _ => None,
    }
}

pub fn for_each_answer_stats(
    practiced_entry: &PracticedWordEntryResult,
    mut f: impl FnMut(&'static str, &AnswerStats),
) {
    for key in ANSWER_FIELD_OPTIONS {
        if let Some(stats) = answer_stats_ref(practiced_entry, key) {
            f(key, stats);
        }
    }
}

pub fn for_each_answer_stats_pair_mut(
    target: &mut PracticedWordEntryResult,
    source: &PracticedWordEntryResult,
    mut f: impl FnMut(&'static str, &mut AnswerStats, &AnswerStats),
) {
    for key in ANSWER_FIELD_OPTIONS {
        if let (Some(target_stats), Some(source_stats)) =
            (answer_stats_mut(target, key), answer_stats_ref(source, key))
        {
            f(key, target_stats, source_stats);
        }
    }
}
