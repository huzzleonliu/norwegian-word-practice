//! i18n 工具：中英文双语文案切换与字段显示名映射。

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
