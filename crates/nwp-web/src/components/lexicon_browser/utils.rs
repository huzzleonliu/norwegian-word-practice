//! 词库浏览器工具集：列定义、筛选匹配、排序比较与输入转换。

use std::cmp::Ordering;

pub use crate::structures::field_meta::DATA_COLUMN_KEYS;
use crate::structures::field_meta::{
    default_search_column_visibility as default_search_column_visibility_from_meta,
    entry_field_value, field_label,
};
use crate::structures::word_bank_entry::{UiLanguage, WordBankEntry};
use crate::utils::i18n::{normalize_for_compare, tr};

pub fn default_search_column_visibility() -> Vec<bool> {
    default_search_column_visibility_from_meta()
}

/// 解析以 `|` 分隔的多值文本（用于 tags/english/chinese）。
pub fn parse_pipe_list(raw: &str) -> Vec<String> {
    raw.split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn compare_entries_by_rules(
    a: &WordBankEntry,
    b: &WordBankEntry,
    rules: &[(usize, bool)],
) -> Ordering {
    for (col_idx, asc) in rules {
        let ordering = compare_entries_by_column(a, b, *col_idx, *asc);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.id.cmp(&b.id)
}

pub(super) fn entry_matches_filter(entry: &WordBankEntry, query: &str, columns: &[bool]) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let query_normalized = normalize_for_compare(trimmed);
    if query_normalized.is_empty() {
        return true;
    }

    (0..DATA_COLUMN_KEYS.len()).any(|idx| {
        columns.get(idx).copied().unwrap_or(false)
            && normalize_for_compare(&column_search_text(entry, idx)).contains(&query_normalized)
    })
}

pub(super) fn header_name(col_idx: usize) -> &'static str {
    DATA_COLUMN_KEYS.get(col_idx).copied().unwrap_or("unknown")
}

pub(super) fn format_sort_rules(rules: &[(usize, bool)], lang: UiLanguage) -> String {
    if rules.is_empty() {
        return tr(lang, "未应用排序", "No sort applied").to_string();
    }

    rules
        .iter()
        .enumerate()
        .map(|(idx, (col, asc))| {
            format!(
                "{}. {}{}",
                idx + 1,
                field_label(lang, header_name(*col)),
                if *asc { "↑" } else { "↓" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn input_to_option(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn compare_entries_by_column(
    a: &WordBankEntry,
    b: &WordBankEntry,
    col_idx: usize,
    ascending: bool,
) -> Ordering {
    let ordering = column_value_text(a, col_idx).cmp(&column_value_text(b, col_idx));

    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

fn column_value_text(entry: &WordBankEntry, col_idx: usize) -> String {
    DATA_COLUMN_KEYS
        .get(col_idx)
        .map(|key| entry_field_value(entry, key))
        .unwrap_or_default()
}

/// 搜索用列文本：词性同时包含稳定 key 与中/英文显示名，便于按界面语言筛选。
fn column_search_text(entry: &WordBankEntry, col_idx: usize) -> String {
    let Some(key) = DATA_COLUMN_KEYS.get(col_idx).copied() else {
        return String::new();
    };
    if key == "part_of_speech" {
        format!(
            "{} {} {}",
            entry.part_of_speech.as_key(),
            entry.part_of_speech.display_name(UiLanguage::Zh),
            entry.part_of_speech.display_name(UiLanguage::En),
        )
    } else {
        entry_field_value(entry, key)
    }
}
