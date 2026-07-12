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
        "verb_present_participle" => tr(lang, "动词_现在分词", "verb_present_participle"),
        "verb_past_participle" => tr(lang, "动词_过去分词", "verb_past_participle"),
        "verb_passive_infinitive" => tr(lang, "动词_被动不定式", "verb_passive_infinitive"),
        "verb_passive_present" => tr(lang, "动词_被动现在时", "verb_passive_present"),
        "verb_passive_past" => tr(lang, "动词_被动过去时", "verb_passive_past"),
        "noun_plural" => tr(lang, "名词_复数", "noun_plural"),
        "noun_singular_definite" => tr(lang, "名词_单数特指", "noun_singular_definite"),
        "noun_plural_definite" => tr(lang, "名词_复数特指", "noun_plural_definite"),
        "noun_singular_definite_genitive" => tr(
            lang,
            "名词_单数特指所有格",
            "noun_singular_definite_genitive",
        ),
        "noun_plural_definite_genitive" => {
            tr(lang, "名词_复数特指所有格", "noun_plural_definite_genitive")
        }
        "noun_singular_indefinite_genitive" => tr(
            lang,
            "名词_单数泛指所有格",
            "noun_singular_indefinite_genitive",
        ),
        "noun_plural_indefinite_genitive" => tr(
            lang,
            "名词_复数泛指所有格",
            "noun_plural_indefinite_genitive",
        ),
        "adjective_feminine_form" => tr(lang, "形容词_阴性", "adjective_feminine_form"),
        "adjective_neuter_form" => tr(lang, "形容词_中性", "adjective_neuter_form"),
        "adjective_plural_form" => tr(lang, "形容词_复数", "adjective_plural_form"),
        "adjective_comparative" => tr(lang, "形容词_比较级", "adjective_comparative"),
        "adjective_superlative_indefinite" => tr(
            lang,
            "形容词_最高级泛指",
            "adjective_superlative_indefinite",
        ),
        "adjective_superlative_definite" => {
            tr(lang, "形容词_最高级特指", "adjective_superlative_definite")
        }
        "pronoun_object" => tr(lang, "代词_宾格", "pronoun_object"),
        "pronoun_reflexive" => tr(lang, "代词_反身", "pronoun_reflexive"),
        "pronoun_plural_subject" => tr(lang, "代词_复数主格", "pronoun_plural_subject"),
        "pronoun_plural_object" => tr(lang, "代词_复数宾格", "pronoun_plural_object"),
        "pronoun_plural_reflexive" => tr(lang, "代词_复数反身", "pronoun_plural_reflexive"),
        "determinative_feminine_form" => tr(lang, "限定词_阴性", "determinative_feminine_form"),
        "determinative_neuter_form" => tr(lang, "限定词_中性", "determinative_neuter_form"),
        "determinative_plural_form" => tr(lang, "限定词_复数", "determinative_plural_form"),
        "adverb_comparative" => tr(lang, "副词_比较级", "adverb_comparative"),
        "adverb_superlative" => tr(lang, "副词_最高级", "adverb_superlative"),
        _ => "unknown",
    }
}
