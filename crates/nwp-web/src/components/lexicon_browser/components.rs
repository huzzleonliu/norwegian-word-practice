//! 词库浏览器编排层：维护草稿状态、搜索排序、批量选择与确认提交流程。

use std::sync::{Arc, Mutex};

use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use leptos::ev;

use super::commit_service::{CommitError, CommitRequest, prepare_commit};
use super::search_panel::LexiconSearchPanel;
use super::structures::LexiconBrowserMode;
use super::table_panel::LexiconTablePanel;
use super::utils::{
    DATA_COLUMN_KEYS, compare_entries_by_rules, default_search_column_visibility,
    entry_matches_filter, format_sort_rules,
};
use crate::app_state::UiState;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::tr;

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordBankEntry>>,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
    mode: LexiconBrowserMode,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();
    let is_query_mode = mode == LexiconBrowserMode::Query;
    let (col_widths, set_col_widths) = signal({
        let mut widths = vec![160_u16; DATA_COLUMN_COUNT + 1];
        if let Some(selected_col) = widths.get_mut(1) {
            *selected_col = 90;
        }
        if let Some(operation_col) = widths.get_mut(DATA_COLUMN_COUNT) {
            *operation_col = 220;
        }
        widths
    });
    let (baseline_entries, set_baseline_entries) = signal(entries.get_untracked());
    let (draft_entries, set_draft_entries) = signal(entries.get_untracked());
    let (row_undo, set_row_undo) =
        signal(vec![None::<WordBankEntry>; entries.get_untracked().len()]);
    let (resize_state, set_resize_state) = signal(None::<(usize, i32, u16)>);
    let (selection_drag_target, set_selection_drag_target) = signal(None::<bool>);
    let (delete_marks, set_delete_marks) = signal(vec![false; entries.get_untracked().len()]);
    let (delete_drag_target, set_delete_drag_target) = signal(None::<bool>);
    let (sort_state, set_sort_state) = signal(Vec::<(usize, bool)>::new());
    let (search_text, set_search_text) = signal(String::new());
    let (search_query, set_search_query) = signal(String::new());
    let (search_columns, set_search_columns) = signal(default_search_column_visibility());
    let (search_scope_expanded, set_search_scope_expanded) = signal(false);
    let (confirm_error, set_confirm_error) = signal(String::new());
    let (confirm_success, set_confirm_success) = signal(String::new());
    let (hash_dirty_entry_ids, set_hash_dirty_entry_ids) = signal(Vec::<String>::new());
    // `set_entries` 被重定向为草稿写接口；真正提交时使用 `set_committed_entries`。
    let set_committed_entries = set_entries;
    let set_entries = set_draft_entries;

    // 当全局词库版本变化时，重置浏览器内部草稿与辅助状态。
    Effect::new(move |_| {
        let _ = data_version.get();
        let current = entries.get();
        set_baseline_entries.set(current.clone());
        set_draft_entries.set(current.clone());
        set_row_undo.set(vec![None; current.len()]);
        set_delete_marks.set(vec![false; current.len()]);
        set_hash_dirty_entry_ids.set(Vec::new());
        set_confirm_error.set(String::new());
        set_confirm_success.set(String::new());
    });

    let row_items = Signal::derive(move || {
        let query = search_query.get();
        let columns = search_columns.get();
        draft_entries
            .get()
            .into_iter()
            .enumerate()
            .filter(|(_, entry)| entry_matches_filter(entry, &query, &columns))
            .collect::<Vec<(usize, WordBankEntry)>>()
    });
    let min_width_for = move |idx: usize| -> u16 {
        match idx {
            1 => 70,
            idx if idx == DATA_COLUMN_COUNT => 260,
            _ => 100,
        }
    };
    let resize_move_listener = Arc::new(Mutex::new(None::<WindowListenerHandle>));
    let resize_up_listener = Arc::new(Mutex::new(None::<WindowListenerHandle>));
    let start_resize = {
        let resize_move_listener = Arc::clone(&resize_move_listener);
        let resize_up_listener = Arc::clone(&resize_up_listener);
        move |idx: usize, ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
            let start_x = ev.client_x();
            let start_w = col_widths.get_untracked().get(idx).copied().unwrap_or(120);
            set_resize_state.set(Some((idx, start_x, start_w)));

            #[cfg(target_arch = "wasm32")]
            {
                if let Some(handle) = resize_move_listener
                    .lock()
                    .expect("resize move listener lock")
                    .take()
                {
                    handle.remove();
                }
                if let Some(handle) = resize_up_listener
                    .lock()
                    .expect("resize up listener lock")
                    .take()
                {
                    handle.remove();
                }

                let move_handle =
                    window_event_listener(ev::mousemove, move |ev: web_sys::MouseEvent| {
                        if let Some((idx, start_x, start_w)) = resize_state.get_untracked() {
                            let delta = ev.client_x() - start_x;
                            let min = i32::from(min_width_for(idx));
                            let next = (i32::from(start_w) + delta).max(min).min(900) as u16;
                            set_col_widths.update(|cols| {
                                if let Some(col) = cols.get_mut(idx) {
                                    *col = next;
                                }
                            });
                        }
                    });
                *resize_move_listener
                    .lock()
                    .expect("resize move listener set") = Some(move_handle);

                let resize_move_listener_for_up = Arc::clone(&resize_move_listener);
                let resize_up_listener_for_up = Arc::clone(&resize_up_listener);
                let up_handle =
                    window_event_listener(ev::mouseup, move |_ev: web_sys::MouseEvent| {
                        set_resize_state.set(None);
                        if let Some(handle) = resize_move_listener_for_up
                            .lock()
                            .expect("resize move up lock")
                            .take()
                        {
                            handle.remove();
                        }
                        if let Some(handle) = resize_up_listener_for_up
                            .lock()
                            .expect("resize up up lock")
                            .take()
                        {
                            handle.remove();
                        }
                    });
                *resize_up_listener.lock().expect("resize up listener set") = Some(up_handle);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (
                    &resize_move_listener,
                    &resize_up_listener,
                    &set_col_widths,
                    &resize_state,
                    min_width_for(idx),
                );
            }
        }
    };
    // 非 wasm / 兜底：若 window 监听不可用，仍允许在面板内拖拽改列宽。
    let on_mouse_move = move |ev: leptos::ev::MouseEvent| {
        if let Some((idx, start_x, start_w)) = resize_state.get_untracked() {
            let delta = ev.client_x() - start_x;
            let min = i32::from(min_width_for(idx));
            let next = (i32::from(start_w) + delta).max(min).min(900) as u16;
            set_col_widths.update(|cols| {
                if let Some(col) = cols.get_mut(idx) {
                    *col = next;
                }
            });
        }
    };
    let stop_pointer_drags = move |_| {
        if selection_drag_target.get_untracked().is_some() {
            set_selection_drag_target.set(None);
        }
        if delete_drag_target.get_untracked().is_some() {
            set_delete_drag_target.set(None);
        }
    };
    let stop_all_drags = move |_| {
        if resize_state.get_untracked().is_some() {
            set_resize_state.set(None);
        }
        if selection_drag_target.get_untracked().is_some() {
            set_selection_drag_target.set(None);
        }
        if delete_drag_target.get_untracked().is_some() {
            set_delete_drag_target.set(None);
        }
    };
    on_cleanup({
        let resize_move_listener = Arc::clone(&resize_move_listener);
        let resize_up_listener = Arc::clone(&resize_up_listener);
        move || {
            if let Some(handle) = resize_move_listener
                .lock()
                .expect("resize move cleanup lock")
                .take()
            {
                handle.remove();
            }
            if let Some(handle) = resize_up_listener
                .lock()
                .expect("resize up cleanup lock")
                .take()
            {
                handle.remove();
            }
        }
    });
    let begin_selected_drag = move |idx: usize, current_selected: bool| {
        let target_checked = !current_selected;
        set_selection_drag_target.set(Some(target_checked));
        update_selected_cell(idx, target_checked, set_entries, set_row_undo);
    };
    let drag_over_selected = move |idx: usize, ev: leptos::ev::MouseEvent| {
        if ev.buttons() & 1 != 1 {
            return;
        }
        if let Some(target_checked) = selection_drag_target.get_untracked() {
            update_selected_cell(idx, target_checked, set_entries, set_row_undo);
        }
    };
    let begin_delete_drag = move |idx: usize, current_checked: bool| {
        let target_checked = !current_checked;
        set_delete_drag_target.set(Some(target_checked));
        update_delete_mark(idx, target_checked, set_delete_marks);
    };
    let drag_over_delete = move |idx: usize, ev: leptos::ev::MouseEvent| {
        if ev.buttons() & 1 != 1 {
            return;
        }
        if let Some(target_checked) = delete_drag_target.get_untracked() {
            update_delete_mark(idx, target_checked, set_delete_marks);
        }
    };
    let sort_by_column = move |col_idx: usize, with_secondary: bool| {
        if col_idx >= DATA_COLUMN_COUNT {
            return;
        }

        let mut sort_rules = sort_state.get_untracked();
        if with_secondary {
            if let Some((_, asc)) = sort_rules.iter_mut().find(|(idx, _)| *idx == col_idx) {
                *asc = !*asc;
            } else {
                sort_rules.push((col_idx, true));
            }

            if sort_rules.len() > 2 {
                let primary = sort_rules[0];
                let secondary = *sort_rules.last().unwrap_or(&primary);
                sort_rules = vec![primary, secondary];
            }
        } else if let Some((idx, asc)) = sort_rules.first().copied() {
            if idx == col_idx {
                sort_rules = vec![(idx, !asc)];
            } else {
                sort_rules = vec![(col_idx, true)];
            }
        } else {
            sort_rules = vec![(col_idx, true)];
        }

        let current_entries = draft_entries.get_untracked();
        if current_entries.len() <= 1 {
            set_sort_state.set(sort_rules);
            return;
        }

        let current_baseline = baseline_entries.get_untracked();
        let current_undo = row_undo.get_untracked();
        let current_delete_marks = delete_marks.get_untracked();

        let mut order: Vec<usize> = (0..current_entries.len()).collect();
        order.sort_by(|a, b| {
            compare_entries_by_rules(&current_entries[*a], &current_entries[*b], &sort_rules)
        });

        let sorted_entries = order
            .iter()
            .filter_map(|idx| current_entries.get(*idx).cloned())
            .collect::<Vec<_>>();
        let sorted_baseline = order
            .iter()
            .map(|idx| {
                current_baseline
                    .get(*idx)
                    .cloned()
                    .unwrap_or_else(|| current_entries[*idx].clone())
            })
            .collect::<Vec<_>>();
        let sorted_undo = order
            .iter()
            .map(|idx| current_undo.get(*idx).cloned().unwrap_or(None))
            .collect::<Vec<_>>();
        let sorted_delete_marks = order
            .iter()
            .map(|idx| current_delete_marks.get(*idx).copied().unwrap_or(false))
            .collect::<Vec<_>>();

        set_draft_entries.set(sorted_entries);
        set_baseline_entries.set(sorted_baseline);
        set_row_undo.set(sorted_undo);
        set_delete_marks.set(sorted_delete_marks);
        set_sort_state.set(sort_rules.clone());
        set_status.set(format!(
            "{} {}",
            tr(lang.get_untracked(), "已排序：", "Sorted:"),
            format_sort_rules(&sort_rules, lang.get_untracked())
        ));
    };
    let apply_search = move || {
        let query = search_text.get().trim().to_string();
        let columns = search_columns.get_untracked();
        set_search_query.set(query.clone());

        if query.is_empty() {
            set_status.set(format!(
                "{} {} {}",
                tr(
                    lang.get_untracked(),
                    "已显示全部条目，共",
                    "Showing all entries,"
                ),
                draft_entries.get_untracked().len(),
                tr(lang.get_untracked(), "条。", "entries.")
            ));
            return;
        }

        let count = draft_entries
            .get_untracked()
            .iter()
            .filter(|entry| entry_matches_filter(entry, &query, &columns))
            .count();
        set_status.set(format!(
            "{} {} {}",
            tr(
                lang.get_untracked(),
                "查找完成：匹配",
                "Search complete: matched"
            ),
            count,
            tr(lang.get_untracked(), "条。", "entries.")
        ));
    };
    let mark_hash_dirty = move |entry_id: String| {
        if entry_id.trim().is_empty() {
            return;
        }
        set_hash_dirty_entry_ids.update(|ids| {
            if !ids.iter().any(|id| id == &entry_id) {
                ids.push(entry_id);
            }
        });
    };
    let confirm_changes = move || {
        set_confirm_error.set(String::new());
        set_confirm_success.set(String::new());

        // Step 1-3) 提交流程下沉到纯函数服务层，组件只处理输入输出与 UI 状态。
        let commit_result = prepare_commit(CommitRequest {
            entries: draft_entries.get_untracked(),
            delete_marks: delete_marks.get_untracked(),
            dirty_entry_ids: hash_dirty_entry_ids.get_untracked(),
            is_query_mode,
        });
        let (entries_to_commit, deleted_count, recalculated_count, removed_duplicate_count) =
            match commit_result {
                Ok(result) => (
                    result.committed_entries,
                    result.summary.deleted_count,
                    result.summary.recalculated_count,
                    result.summary.removed_duplicate_count,
                ),
                Err(CommitError::RecomputedIdConflicts { conflicts }) => {
                    let message = conflicts.join("；");
                    set_confirm_error.set(message.clone());
                    set_status.set(format!(
                        "{} {}",
                        tr(
                            lang.get_untracked(),
                            "确认失败：重算后的 id 与现有词条冲突。",
                            "Confirm failed: recomputed IDs conflict with existing entries.",
                        ),
                        message
                    ));
                    return;
                }
            };

        // Step 4) 一次性提交并同步本地草稿快照。
        set_committed_entries.set(entries_to_commit.clone());
        set_draft_entries.set(entries_to_commit.clone());
        set_baseline_entries.set(entries_to_commit.clone());
        set_row_undo.set(vec![None; entries_to_commit.len()]);
        set_delete_marks.set(vec![false; entries_to_commit.len()]);
        set_hash_dirty_entry_ids.set(Vec::new());
        set_confirm_success.set(tr(lang.get_untracked(), "修改成功", "Saved").to_string());
        if is_query_mode {
            set_status.set(
                tr(
                    lang.get_untracked(),
                    "词库选择已确认并应用。",
                    "Lexicon selection confirmed and applied.",
                )
                .to_string(),
            );
        } else {
            let mut log_items = Vec::<String>::new();
            if deleted_count > 0 {
                log_items.push(format!(
                    "{} {} {}",
                    tr(lang.get_untracked(), "已删除", "deleted"),
                    deleted_count,
                    tr(lang.get_untracked(), "条", "entries")
                ));
            }
            if recalculated_count > 0 {
                log_items.push(format!(
                    "{} {} {}",
                    tr(lang.get_untracked(), "重算 id", "recomputed IDs for"),
                    recalculated_count,
                    tr(lang.get_untracked(), "条", "entries")
                ));
            }
            if removed_duplicate_count > 0 {
                log_items.push(format!(
                    "{} {} {}",
                    tr(
                        lang.get_untracked(),
                        "自动移除重复",
                        "auto-removed duplicate"
                    ),
                    removed_duplicate_count,
                    tr(lang.get_untracked(), "条", "entries")
                ));
            }

            if log_items.is_empty() {
                set_status.set(
                    tr(
                        lang.get_untracked(),
                        "词库修改已确认并应用。",
                        "Lexicon changes confirmed and applied.",
                    )
                    .to_string(),
                );
            } else {
                set_status.set(format!(
                    "{}：{}。",
                    tr(
                        lang.get_untracked(),
                        "词库修改已确认并应用",
                        "Lexicon changes confirmed and applied",
                    ),
                    log_items.join("，"),
                ));
            }
        }
    };
    let set_visible_selected = move |checked: bool| {
        let query = search_query.get_untracked();
        let columns = search_columns.get_untracked();
        let current = draft_entries.get_untracked();
        let visible_indices = current
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                if entry_matches_filter(entry, &query, &columns) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if visible_indices.is_empty() {
            set_status.set(
                tr(
                    lang.get_untracked(),
                    "当前没有可操作的显示条目。",
                    "No visible entries to operate on.",
                )
                .to_string(),
            );
            return;
        }

        let snapshots = visible_indices
            .iter()
            .filter_map(|idx| current.get(*idx).cloned().map(|entry| (*idx, entry)))
            .collect::<Vec<_>>();
        set_row_undo.update(|undo| {
            if undo.len() < current.len() {
                undo.resize(current.len(), None);
            }
            for (idx, entry) in &snapshots {
                undo[*idx] = Some(entry.clone());
            }
        });

        set_entries.update(|list| {
            for idx in &visible_indices {
                if let Some(item) = list.get_mut(*idx) {
                    item.selected = checked;
                }
            }
        });
        set_status.set(format!(
            "{} {} {} {}",
            tr(lang.get_untracked(), "已将当前显示的", "Set"),
            visible_indices.len(),
            tr(lang.get_untracked(), "条词条设置为", "visible entries to"),
            if checked {
                tr(lang.get_untracked(), "选中。", "selected.")
            } else {
                tr(lang.get_untracked(), "不选中。", "unselected.")
            }
        ));
    };
    let select_visible_click = move || set_visible_selected(true);
    let unselect_visible_click = move || set_visible_selected(false);

    let on_apply_search = Callback::new(move |_| apply_search());
    let on_sort_by_column =
        Callback::new(move |(col_idx, with_secondary)| sort_by_column(col_idx, with_secondary));
    let on_start_resize =
        Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| start_resize(idx, ev));
    let on_begin_selected_drag =
        Callback::new(move |(idx, current_selected)| begin_selected_drag(idx, current_selected));
    let on_drag_over_selected = Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| {
        drag_over_selected(idx, ev)
    });
    let on_begin_delete_drag =
        Callback::new(move |(idx, current_checked)| begin_delete_drag(idx, current_checked));
    let on_drag_over_delete =
        Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| drag_over_delete(idx, ev));
    let on_mark_hash_dirty = Callback::new(move |entry_id: String| mark_hash_dirty(entry_id));
    let on_select_visible_click = Callback::new(move |_| select_visible_click());
    let on_unselect_visible_click = Callback::new(move |_| unselect_visible_click());
    let on_confirm_changes = Callback::new(move |_| confirm_changes());

    view! {
            <section
                class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-3 sm:p-4"
                on:mousemove=on_mouse_move
                on:mouseup=stop_all_drags
                on:mouseleave=stop_pointer_drags
            >
                            <LexiconSearchPanel
                    lang=lang
                    search_text=search_text
                    set_search_text=set_search_text
                    search_columns=search_columns
                    set_search_columns=set_search_columns
                    search_scope_expanded=search_scope_expanded
                    set_search_scope_expanded=set_search_scope_expanded
                    on_apply_search=on_apply_search
                />
                <LexiconTablePanel
                    lang=lang
                    is_query_mode=is_query_mode
                    col_widths=col_widths
                    sort_state=sort_state
                    row_items=row_items
                    entries=entries
                    on_sort_by_column=on_sort_by_column
                    on_start_resize=on_start_resize
                    on_begin_selected_drag=on_begin_selected_drag
                    on_drag_over_selected=on_drag_over_selected
                    set_entries=set_entries
                    set_row_undo=set_row_undo
                    baseline_entries=baseline_entries
                    set_baseline_entries=set_baseline_entries
                    row_undo=row_undo
                    set_status=set_status
                    confirm_error=confirm_error
                    confirm_success=confirm_success
                    delete_marks=delete_marks
                    on_begin_delete_drag=on_begin_delete_drag
                    on_drag_over_delete=on_drag_over_delete
                    on_mark_hash_dirty=on_mark_hash_dirty
                    on_select_visible_click=on_select_visible_click
                    on_unselect_visible_click=on_unselect_visible_click
                    on_confirm_changes=on_confirm_changes
                />
    </section>
        }
}

fn update_selected_cell(
    idx: usize,
    checked: bool,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_row_undo: WriteSignal<Vec<Option<WordBankEntry>>>,
) {
    // 记录行级撤销快照后再写入 selected，保证“逐行撤销”可用。
    set_entries.update(|list| {
        if let Some(item) = list.get_mut(idx) {
            if item.selected == checked {
                return;
            }
            let snapshot = item.clone();
            set_row_undo.update(|undo| {
                if undo.len() <= idx {
                    undo.resize(idx + 1, None);
                }
                undo[idx] = Some(snapshot);
            });
            item.selected = checked;
        }
    });
}

fn update_delete_mark(idx: usize, checked: bool, set_delete_marks: WriteSignal<Vec<bool>>) {
    // 删除标记是独立状态，不直接改动词条本体；最终在确认阶段统一生效。
    set_delete_marks.update(|marks| {
        if marks.len() <= idx {
            marks.resize(idx + 1, false);
        }
        marks[idx] = checked;
    });
}
