//! i18n 工具：中英文双语文案切换、字段显示名映射与文本规范化。

use crate::structures::word_bank_entry::UiLanguage;

/// 双语文案选择器：按当前 UI 语言返回中文或英文字符串。
pub fn tr<'a>(lang: UiLanguage, zh: &'a str, en: &'a str) -> &'a str {
    match lang {
        UiLanguage::Zh => zh,
        UiLanguage::En => en,
    }
}

/// 字段 key 到显示文案的映射表（中英双语）。
pub fn field_label(lang: UiLanguage, key: &str) -> &'static str {
    crate::structures::field_meta::field_label(lang, key)
}

/// 文本规范化：小写 + 去空白 + `|` 分段规整 + 挪威字母别名映射。
///
/// 用于判题与搜索，使 `ae`/`oe`/`aa` 可匹配 `æ`/`ø`/`å`。
pub fn normalize_for_compare(value: &str) -> String {
    value
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("|")
        .to_lowercase()
        .replace('æ', "ae")
        .replace('ø', "oe")
        .replace('å', "aa")
}
