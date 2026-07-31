//! 练习引擎纯函数工具：
//! - 判题计算（不触发 UI）
//! - 题目补位与完成迁移
//! - 输入/显隐状态清理

use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use super::buttons::{normalize_for_compare, record_field_check_result};
use super::entry::{
    answer_input_key, build_question_items, entry_field_value, is_answer_field_available,
};
use crate::structures::pracresult::PracticeResult;
use crate::structures::word_bank_entry::WordBankEntry;

#[derive(Clone, Debug)]
pub struct FieldCheckResult {
    pub entry_id: String,
    pub field: String,
    pub is_correct: bool,
    pub actual: String,
}

#[derive(Clone, Debug)]
pub struct CheckAnswersResult {
    pub field_results: Vec<FieldCheckResult>,
    pub newly_solved_ids: Vec<String>,
    pub total_fields: usize,
    pub correct_fields: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckAnswersError {
    NoAnswerFields,
    NoQuestions,
    NoAnswerableFields,
}

/// 判题核心：
/// - 读取 active 题目
/// - 按选中回答字段逐项比较
/// - 统计字段正确率与“整题全对”列表
pub fn evaluate_check_answers(
    selected_answer_fields: &[String],
    active_ids: &[String],
    entries: &[WordBankEntry],
    answers: &HashMap<String, String>,
) -> Result<CheckAnswersResult, CheckAnswersError> {
    if selected_answer_fields.is_empty() {
        return Err(CheckAnswersError::NoAnswerFields);
    }

    let current_questions = build_question_items(active_ids, entries);
    if current_questions.is_empty() {
        return Err(CheckAnswersError::NoQuestions);
    }

    let mut field_results = Vec::<FieldCheckResult>::new();
    let mut newly_solved_ids = Vec::<String>::new();
    let mut total_fields = 0_usize;
    let mut correct_fields = 0_usize;

    for (_, entry) in &current_questions {
        let mut all_correct_for_entry = true;
        let mut checked_any_field = false;
        for field in selected_answer_fields {
            if !is_answer_field_available(entry, field) {
                continue;
            }
            checked_any_field = true;

            let expected = entry_field_value(entry, field);
            let key = answer_input_key(&entry.id, field);
            let actual = answers.get(&key).cloned().unwrap_or_default();
            let is_correct = normalize_for_compare(&actual) == normalize_for_compare(&expected);

            total_fields += 1;
            if is_correct {
                correct_fields += 1;
            } else {
                all_correct_for_entry = false;
            }

            field_results.push(FieldCheckResult {
                entry_id: entry.id.clone(),
                field: field.clone(),
                is_correct,
                actual,
            });
        }

        if checked_any_field && all_correct_for_entry {
            newly_solved_ids.push(entry.id.clone());
        }
    }

    if total_fields == 0 {
        return Err(CheckAnswersError::NoAnswerableFields);
    }

    Ok(CheckAnswersResult {
        field_results,
        newly_solved_ids,
        total_fields,
        correct_fields,
    })
}

/// 把判题结果写回临时统计对象（纯数据更新，不触发状态管理）。
pub fn apply_check_results(temp_result: &mut PracticeResult, field_results: &[FieldCheckResult]) {
    for result in field_results {
        record_field_check_result(
            temp_result,
            &result.entry_id,
            &result.field,
            result.is_correct,
            &result.actual,
        );
    }
}

/// 把本次新增“整题通过”条目合并到 solved 列表（去重）。
pub fn merge_solved_question_ids(solved_ids: &mut Vec<String>, newly_solved_ids: &[String]) {
    for entry_id in newly_solved_ids {
        if !solved_ids.iter().any(|existing| existing == entry_id) {
            solved_ids.push(entry_id.clone());
        }
    }
}

/// 清理已完成题目的输入缓存，避免已通过题目残留文本污染下一轮。
#[allow(dead_code)]
pub fn retain_unsolved_answer_inputs(
    inputs: &mut HashMap<String, String>,
    newly_solved_ids: &[String],
) {
    let solved_set = newly_solved_ids
        .iter()
        .cloned()
        .collect::<HashSet<String>>();
    inputs.retain(|key, _| {
        !solved_set
            .iter()
            .any(|entry_id| key.starts_with(&format!("{entry_id}::")))
    });
}

/// 检查后：清空输入，并为未整题通过的错误字段生成“错误拼写”反馈。
pub fn prepare_post_check_input_state(
    field_results: &[FieldCheckResult],
    newly_solved_ids: &[String],
) -> (HashMap<String, String>, HashMap<String, String>) {
    let solved_set = newly_solved_ids
        .iter()
        .cloned()
        .collect::<HashSet<String>>();
    let mut wrong_feedback = HashMap::<String, String>::new();
    for result in field_results {
        if result.is_correct || solved_set.contains(&result.entry_id) {
            continue;
        }
        let key = answer_input_key(&result.entry_id, &result.field);
        // 空答案也记一笔，便于标红输入框
        wrong_feedback.insert(key, result.actual.clone());
    }
    (HashMap::new(), wrong_feedback)
}

/// 清理已完成题目的错误拼写反馈。
#[allow(dead_code)]
pub fn retain_unsolved_wrong_feedback(
    feedback: &mut HashMap<String, String>,
    newly_solved_ids: &[String],
) {
    let solved_set = newly_solved_ids
        .iter()
        .cloned()
        .collect::<HashSet<String>>();
    feedback.retain(|key, _| {
        !solved_set
            .iter()
            .any(|entry_id| key.starts_with(&format!("{entry_id}::")))
    });
}

/// 清理已完成题目的“显示答案”状态。
pub fn retain_unsolved_revealed_keys(keys: &mut HashSet<String>, newly_solved_ids: &[String]) {
    let solved_set = newly_solved_ids
        .iter()
        .cloned()
        .collect::<HashSet<String>>();
    keys.retain(|key| {
        !solved_set
            .iter()
            .any(|entry_id| key.starts_with(&format!("{entry_id}::")))
    });
}

/// 重置一轮练习的页面局部状态（不影响全局累计结果）。
pub fn clear_practice_round_local_state(
    set_solved_question_ids: WriteSignal<Vec<String>>,
    set_answer_inputs: WriteSignal<HashMap<String, String>>,
    set_active_question_ids: WriteSignal<Vec<String>>,
    set_revealed_answer_keys: Option<WriteSignal<HashSet<String>>>,
) {
    set_solved_question_ids.set(Vec::new());
    set_answer_inputs.set(HashMap::new());
    set_active_question_ids.set(Vec::new());
    if let Some(set_revealed_answer_keys) = set_revealed_answer_keys {
        set_revealed_answer_keys.set(HashSet::new());
    }
}

/// 保留未解题并按 selected 顺序补满 active 列表。
pub fn refill_active_question_ids(
    selected_ids: &[String],
    current_active_ids: &[String],
    solved_ids: &[String],
    page_size: usize,
) -> Vec<String> {
    let solved_set = solved_ids.iter().cloned().collect::<HashSet<String>>();
    let mut next_active = current_active_ids
        .iter()
        .filter(|id| !solved_set.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let mut used = next_active.iter().cloned().collect::<HashSet<String>>();

    for id in selected_ids {
        if next_active.len() >= page_size {
            break;
        }
        if solved_set.contains(id) || used.contains(id) {
            continue;
        }
        next_active.push(id.clone());
        used.insert(id.clone());
    }

    next_active.truncate(page_size.min(selected_ids.len()));
    next_active
}
