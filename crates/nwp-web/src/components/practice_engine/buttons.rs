//! 练习流程按钮与结果聚合：
//! - UI 按钮组件（完成/重开/放弃）
//! - 练习结果合并与错法维护
//! - 判题文本规范化（用于输入比较）

use leptos::prelude::*;

use crate::app_state::{NavigateToPage, PracticeState, UiState};
use crate::pages::AppPage;
use crate::structures::field_meta::{answer_stats_mut, for_each_answer_stats_pair_mut};
use crate::structures::pracresult::{
    AnswerStats, MAX_WRONG_ANSWERS_PER_FORM, PracticeResult, PracticedWordEntryResult,
};
use crate::utils::i18n::tr;

pub use crate::utils::i18n::normalize_for_compare;
use crate::utils::shuffle::shuffle_strings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RestartTempBehavior {
    Keep,
    Reinitialize,
}

#[component]
pub fn FinishPracticeButton(
    practice_state: PracticeState,
    set_current_page: NavigateToPage,
    set_status: WriteSignal<String>,
    finish_target_page: AppPage,
    summary_return_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let finish_click = move |_| {
        practice_state
            .set_summary_return_page
            .set(summary_return_page);
        handle_finish_click(
            practice_state,
            lang.get_untracked(),
            set_current_page,
            set_status,
            finish_target_page,
        );
    };
    let label = label
        .unwrap_or_else(|| tr(lang.get_untracked(), "完成练习", "Finish Practice").to_string());
    let class = class.unwrap_or_else(|| {
        "w-full rounded-lg border border-indigo-600 bg-indigo-700 px-4 py-3 text-sm font-semibold text-white hover:bg-indigo-600 sm:w-auto"
            .to_string()
    });

    view! {
        <button type="button" on:click=finish_click class=class>
            {label}
        </button>
    }
}

#[component]
pub fn RestartPracticeButton(
    practice_state: PracticeState,
    set_status: WriteSignal<String>,
    on_restart_ui: Callback<()>,
    restart_message: String,
    restart_temp_behavior: RestartTempBehavior,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let restart_click = move |_| {
        handle_restart_click(
            practice_state,
            set_status,
            on_restart_ui,
            &restart_message,
            restart_temp_behavior,
        );
    };
    let label =
        label.unwrap_or_else(|| tr(lang.get_untracked(), "重新练习", "Restart").to_string());
    let class = class.unwrap_or_else(|| {
        "w-full rounded-lg border border-amber-600 bg-amber-700 px-4 py-3 text-sm font-semibold text-white hover:bg-amber-600 sm:w-auto"
            .to_string()
    });

    view! {
        <button type="button" on:click=restart_click class=class>
            {label}
        </button>
    }
}

#[component]
pub fn AbortPracticeButton(
    practice_state: PracticeState,
    set_current_page: NavigateToPage,
    abort_target_page: AppPage,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let abort_click = move |_| {
        handle_abort_click(practice_state, set_current_page, abort_target_page);
    };
    let label = label.unwrap_or_else(|| {
        tr(lang.get_untracked(), "放弃练习并返回", "Abort and Return").to_string()
    });
    let class = class.unwrap_or_else(|| {
        "w-full rounded-lg border border-rose-600 bg-rose-700 px-4 py-3 text-sm font-semibold text-white hover:bg-rose-600 sm:w-auto"
            .to_string()
    });

    view! {
        <button type="button" on:click=abort_click class=class>
            {label}
        </button>
    }
}

/// 基于历史结果生成“本轮临时结果壳”，只继承用户与密钥上下文。
pub fn create_temp_practice_result(
    base: &PracticeResult,
    selected_ids: &[String],
) -> PracticeResult {
    PracticeResult {
        username: base.username.clone(),
        encryption_key: base.encryption_key.clone(),
        selected_word_entry_ids: selected_ids.to_vec(),
        practiced_word_entries: Vec::new(),
    }
}

/// 完成练习动作：
/// - 把临时结果合并到历史结果；
/// - 缓存最近完成快照；
/// - 打乱并重置下一轮待练习队列。
pub fn handle_finish_click(
    practice_state: PracticeState,
    lang: crate::structures::word_bank_entry::UiLanguage,
    set_current_page: NavigateToPage,
    set_status: WriteSignal<String>,
    finish_target_page: AppPage,
) {
    let mut selected_ids = practice_state.selected_word_entry_ids.get_untracked();
    if selected_ids.is_empty() {
        set_status.set(
            tr(
                lang,
                "当前没有可练习词条。",
                "No entries available for practice.",
            )
            .to_string(),
        );
        return;
    }

    let temp_snapshot = practice_state.temp_practice_result.get_untracked();
    // 完成时把本轮临时结果合并进历史结果，并保留一份“最近完成快照”给总结页展示。
    practice_state.set_practice_result.update(|global_result| {
        merge_practice_result(global_result, &temp_snapshot);
    });
    practice_state
        .set_last_completed_practice_result
        .set(temp_snapshot.clone());

    shuffle_strings(&mut selected_ids);
    practice_state
        .set_selected_word_entry_ids
        .set(selected_ids.clone());

    let latest_global = practice_state.practice_result.get_untracked();
    practice_state
        .set_temp_practice_result
        .set(create_temp_practice_result(&latest_global, &selected_ids));
    set_current_page.set(finish_target_page);
}

/// 重开动作：可选是否重建临时统计，同时重置页面局部输入状态。
pub fn handle_restart_click(
    practice_state: PracticeState,
    set_status: WriteSignal<String>,
    on_restart_ui: Callback<()>,
    restart_message: &str,
    restart_temp_behavior: RestartTempBehavior,
) {
    if restart_temp_behavior == RestartTempBehavior::Reinitialize {
        let selected_ids = practice_state.selected_word_entry_ids.get_untracked();
        let base_result = practice_state.practice_result.get_untracked();
        practice_state
            .set_temp_practice_result
            .set(create_temp_practice_result(&base_result, &selected_ids));
    }

    on_restart_ui.run(());
    set_status.set(restart_message.to_string());
}

/// 放弃动作：清空临时统计并返回指定页面。
pub fn handle_abort_click(
    practice_state: PracticeState,
    set_current_page: NavigateToPage,
    abort_target_page: AppPage,
) {
    practice_state
        .set_temp_practice_result
        .set(PracticeResult::default());
    set_current_page.set(abort_target_page);
}

/// 记录单字段判题结果并维护错法队列。
pub fn record_field_check_result(
    temp_result: &mut PracticeResult,
    entry_id: &str,
    field: &str,
    is_correct: bool,
    user_input: &str,
) {
    let practiced_entry = get_or_insert_practiced_entry(temp_result, entry_id);
    let Some(stats) = answer_stats_mut(practiced_entry, field) else {
        return;
    };

    if is_correct {
        stats.correct_count += 1;
        return;
    }

    stats.wrong_count += 1;
    let wrong_input = user_input.trim();
    if wrong_input.is_empty() {
        return;
    }
    push_or_promote_wrong_answer(&mut stats.wrong_answers, wrong_input);
}

/// 合并“本轮临时结果”到“全局累计结果”。
pub fn merge_practice_result(global_result: &mut PracticeResult, temp_result: &PracticeResult) {
    if global_result.username.trim().is_empty() && !temp_result.username.trim().is_empty() {
        global_result.username = temp_result.username.clone();
    }
    if global_result.encryption_key.trim().is_empty()
        && !temp_result.encryption_key.trim().is_empty()
    {
        global_result.encryption_key = temp_result.encryption_key.clone();
    }

    for id in &temp_result.selected_word_entry_ids {
        if !global_result
            .selected_word_entry_ids
            .iter()
            .any(|existing| existing == id)
        {
            global_result.selected_word_entry_ids.push(id.clone());
        }
    }

    for temp_entry in &temp_result.practiced_word_entries {
        if let Some(existing_entry) = global_result
            .practiced_word_entries
            .iter_mut()
            .find(|entry| entry.id == temp_entry.id)
        {
            // 同一词条按字段累加 correct/wrong 计数，并维护错法最近队列。
            merge_practiced_entry(existing_entry, temp_entry);
        } else {
            global_result
                .practiced_word_entries
                .push(temp_entry.clone());
        }
    }
}

fn merge_practiced_entry(
    existing_entry: &mut PracticedWordEntryResult,
    temp_entry: &PracticedWordEntryResult,
) {
    for_each_answer_stats_pair_mut(
        existing_entry,
        temp_entry,
        |_, existing_stats, temp_stats| {
            merge_answer_stats(existing_stats, temp_stats);
        },
    );
}

fn merge_answer_stats(existing_stats: &mut AnswerStats, temp_stats: &AnswerStats) {
    existing_stats.correct_count += temp_stats.correct_count;
    existing_stats.wrong_count += temp_stats.wrong_count;

    for wrong in temp_stats.wrong_answers.iter().rev() {
        push_or_promote_wrong_answer(&mut existing_stats.wrong_answers, wrong);
    }
}

fn push_or_promote_wrong_answer(list: &mut Vec<String>, wrong_answer: &str) {
    let normalized = normalize_for_compare(wrong_answer);
    if normalized.is_empty() {
        return;
    }

    if let Some(existing_pos) = list
        .iter()
        .position(|current| normalize_for_compare(current) == normalized)
    {
        list.remove(existing_pos);
    }

    list.insert(0, wrong_answer.trim().to_string());
    if list.len() > MAX_WRONG_ANSWERS_PER_FORM {
        list.truncate(MAX_WRONG_ANSWERS_PER_FORM);
    }
}

fn get_or_insert_practiced_entry<'a>(
    temp_result: &'a mut PracticeResult,
    entry_id: &str,
) -> &'a mut PracticedWordEntryResult {
    if let Some(index) = temp_result
        .practiced_word_entries
        .iter()
        .position(|entry| entry.id == entry_id)
    {
        return &mut temp_result.practiced_word_entries[index];
    }

    let entry = PracticedWordEntryResult {
        id: entry_id.to_string(),
        ..PracticedWordEntryResult::default()
    };
    temp_result.practiced_word_entries.push(entry);
    let last_index = temp_result.practiced_word_entries.len() - 1;
    &mut temp_result.practiced_word_entries[last_index]
}
