//! 词库浏览器提交服务：提供可单测的纯函数提交流程。
//! 负责删除标记处理、脏行 id 重算、内部去重、冲突检测与结果汇总。

use std::collections::{HashMap, HashSet};

use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::dictionary::compute_word_entry_id;

#[derive(Debug, Clone)]
pub struct CommitRequest {
    pub entries: Vec<WordBankEntry>,
    pub delete_marks: Vec<bool>,
    pub dirty_entry_ids: Vec<String>,
    pub is_query_mode: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CommitSummary {
    pub deleted_count: usize,
    pub recalculated_count: usize,
    pub removed_duplicate_count: usize,
}

#[derive(Debug, Clone)]
pub struct CommitResult {
    pub committed_entries: Vec<WordBankEntry>,
    pub summary: CommitSummary,
}

#[derive(Debug, Clone)]
pub enum CommitError {
    RecomputedIdConflicts { conflicts: Vec<String> },
}

pub fn prepare_commit(request: CommitRequest) -> Result<CommitResult, CommitError> {
    let CommitRequest {
        entries,
        delete_marks,
        dirty_entry_ids,
        is_query_mode,
    } = request;

    let mut summary = CommitSummary::default();
    let mut entries_to_commit = if is_query_mode {
        entries
    } else {
        let kept = entries
            .into_iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                if delete_marks.get(idx).copied().unwrap_or(false) {
                    None
                } else {
                    Some(entry)
                }
            })
            .collect::<Vec<_>>();
        summary.deleted_count = delete_marks.iter().filter(|marked| **marked).count();
        kept
    };

    if is_query_mode {
        return Ok(CommitResult {
            committed_entries: entries_to_commit,
            summary,
        });
    }

    let dirty_id_set = dirty_entry_ids.into_iter().collect::<HashSet<_>>();
    if dirty_id_set.is_empty() {
        return Ok(CommitResult {
            committed_entries: entries_to_commit,
            summary,
        });
    }

    let mut changed_rows = Vec::<(usize, String)>::new();
    for (idx, entry) in entries_to_commit.iter().enumerate() {
        if dirty_id_set.contains(&entry.id) {
            changed_rows.push((idx, compute_word_entry_id(entry)));
        }
    }
    summary.recalculated_count = changed_rows.len();

    if changed_rows.is_empty() {
        return Ok(CommitResult {
            committed_entries: entries_to_commit,
            summary,
        });
    }

    let mut first_seen_new_id = HashMap::<String, usize>::new();
    let mut remove_indices = Vec::<usize>::new();
    for (idx, new_id) in &changed_rows {
        if first_seen_new_id.contains_key(new_id) {
            remove_indices.push(*idx);
        } else {
            first_seen_new_id.insert(new_id.clone(), *idx);
        }
    }

    remove_indices.sort_unstable();
    remove_indices.dedup();
    summary.removed_duplicate_count = remove_indices.len();

    for idx in remove_indices.iter().rev() {
        if *idx < entries_to_commit.len() {
            entries_to_commit.remove(*idx);
        }
    }

    let mut changed_after_dedup = Vec::<(usize, String)>::new();
    for (idx, entry) in entries_to_commit.iter().enumerate() {
        if dirty_id_set.contains(&entry.id) {
            changed_after_dedup.push((idx, compute_word_entry_id(entry)));
        }
    }

    let changed_idx_set = changed_after_dedup
        .iter()
        .map(|(idx, _)| *idx)
        .collect::<HashSet<_>>();
    let outside_ids = entries_to_commit
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if changed_idx_set.contains(&idx) {
                None
            } else {
                Some(entry.id.clone())
            }
        })
        .collect::<HashSet<_>>();

    let mut conflicts = Vec::<String>::new();
    for (idx, new_id) in &changed_after_dedup {
        if outside_ids.contains(new_id) {
            conflicts.push(format!("第 {} 行新 id 冲突：{new_id}", idx + 1));
        }
    }
    if !conflicts.is_empty() {
        return Err(CommitError::RecomputedIdConflicts { conflicts });
    }

    for (idx, new_id) in changed_after_dedup {
        if let Some(entry) = entries_to_commit.get_mut(idx) {
            entry.id = new_id;
        }
    }

    Ok(CommitResult {
        committed_entries: entries_to_commit,
        summary,
    })
}
