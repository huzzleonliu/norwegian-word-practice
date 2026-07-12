use crate::structures::word_bank_entry::UiLanguage;

pub fn tr<'a>(lang: UiLanguage, zh: &'a str, en: &'a str) -> &'a str {
    match lang {
        UiLanguage::Zh => zh,
        UiLanguage::En => en,
    }
}

pub fn field_label(lang: UiLanguage, key: &str) -> &'static str {
    match key {
        "none" => tr(lang, "无", "None"),
        "id" => tr(lang, "序号", "ID"),
        "selected" => tr(lang, "选中", "Selected"),
        "part_of_speech" => tr(lang, "词性", "Part of Speech"),
        "tags" => tr(lang, "标签", "Tags"),
        "english" => tr(lang, "英文", "English"),
        "chinese" => tr(lang, "中文", "Chinese"),
        "base_form" => tr(lang, "原型", "Base Form"),
        "verb_present_tense" => tr(lang, "动词_现在时", "verb_present_tense"),
        "verb_past_tense" => tr(lang, "动词_过去式", "verb_past_tense"),
        "verb_imperative" => tr(lang, "动词_祈使式", "verb_imperative"),
        "noun_plural" => tr(lang, "名词_复数", "noun_plural"),
        "noun_singular_definite" => tr(lang, "名词_单数特指", "noun_singular_definite"),
        "noun_plural_definite" => tr(lang, "名词_复数特指", "noun_plural_definite"),
        "adjective_neuter_form" => tr(lang, "形容词_中性", "adjective_neuter_form"),
        "adjective_plural_form" => tr(lang, "形容词_复数", "adjective_plural_form"),
        "adjective_comparative" => tr(lang, "形容词_比较级", "adjective_comparative"),
        "adjective_superlative_indefinite" => {
            tr(lang, "形容词_最高级泛指", "adjective_superlative_indefinite")
        }
        "adjective_superlative_definite" => {
            tr(lang, "形容词_最高级特指", "adjective_superlative_definite")
        }
        "adverb_comparative" => tr(lang, "副词_比较级", "adverb_comparative"),
        "adverb_superlative" => tr(lang, "副词_最高级", "adverb_superlative"),
        _ => "unknown",
    }
}

