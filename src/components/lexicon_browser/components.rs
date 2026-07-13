use leptos::prelude::*;

use super::search_panel::LexiconSearchPanel;
use super::structures::LexiconBrowserMode;
use super::table_panel::LexiconTablePanel;
use super::utils::{
    DATA_COLUMN_KEYS, compare_entries_by_rules, default_search_column_visibility,
    entry_matches_filter, format_sort_rules,
};
use crate::app_state::WordBankState;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::dictionary::validate_and_prepare_all_entries;
use crate::utils::i18n::tr;

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordBankEntry>>,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
    mode: LexiconBrowserMode,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
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
    let (sort_state, set_sort_state) = signal(Vec::<(usize, bool)>::new());
    let (search_text, set_search_text) = signal(String::new());
    let (search_query, set_search_query) = signal(String::new());
    let (search_columns, set_search_columns) = signal(default_search_column_visibility());
    let (search_scope_expanded, set_search_scope_expanded) = signal(false);
    let (confirm_error, set_confirm_error) = signal(String::new());
    let (confirm_success, set_confirm_success) = signal(String::new());
    let set_committed_entries = set_entries;
    let set_entries = set_draft_entries;

    Effect::new(move |_| {
        let _ = data_version.get();
        let current = entries.get();
        set_baseline_entries.set(current.clone());
        set_draft_entries.set(current.clone());
        set_row_undo.set(vec![None; current.len()]);
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
    let min_width_for = |idx: usize| -> u16 {
        match idx {
            1 => 70,
            idx if idx == DATA_COLUMN_COUNT => 260,
            7..=37 => 100,
            _ => 160,
        }
    };
    let start_resize = move |idx: usize, ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let start_x = ev.client_x();
        let start_w = col_widths.get_untracked().get(idx).copied().unwrap_or(120);
        set_resize_state.set(Some((idx, start_x, start_w)));
    };
    let on_mouse_move = move |ev: leptos::ev::MouseEvent| {
        if let Some((idx, start_x, start_w)) = resize_state.get() {
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
    let stop_resize = move |_| {
        if resize_state.get_untracked().is_some() {
            set_resize_state.set(None);
        }
        if selection_drag_target.get_untracked().is_some() {
            set_selection_drag_target.set(None);
        }
    };
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

        set_draft_entries.set(sorted_entries);
        set_baseline_entries.set(sorted_baseline);
        set_row_undo.set(sorted_undo);
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
    let confirm_changes = move || {
        set_confirm_error.set(String::new());
        set_confirm_success.set(String::new());

        let current_entries = draft_entries.get_untracked();
        let mut entries_to_commit = current_entries.clone();

        if !is_query_mode {
            match validate_and_prepare_all_entries(&current_entries) {
                Ok(validated_entries) => entries_to_commit = validated_entries,
                Err(errors) => {
                    let message = errors.join("；");
                    set_confirm_error.set(message.clone());
                    set_status.set(
                        tr(
                            lang.get_untracked(),
                            "确认失败：存在不合法修改。",
                            "Confirm failed: invalid modifications found.",
                        )
                        .to_string(),
                    );
                    return;
                }
            }
        }

        set_committed_entries.set(entries_to_commit.clone());
        set_draft_entries.set(entries_to_commit.clone());
        set_baseline_entries.set(entries_to_commit.clone());
        set_row_undo.set(vec![None; entries_to_commit.len()]);
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
            set_status.set(
                tr(
                    lang.get_untracked(),
                    "词库修改已确认并应用。",
                    "Lexicon changes confirmed and applied.",
                )
                .to_string(),
            );
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
    let on_sort_by_column = Callback::new(move |(col_idx, with_secondary)| sort_by_column(col_idx, with_secondary));
    let on_start_resize = Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| start_resize(idx, ev));
    let on_begin_selected_drag = Callback::new(move |(idx, current_selected)| begin_selected_drag(idx, current_selected));
    let on_drag_over_selected = Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| drag_over_selected(idx, ev));
    let on_select_visible_click = Callback::new(move |_| select_visible_click());
    let on_unselect_visible_click = Callback::new(move |_| unselect_visible_click());
    let on_confirm_changes = Callback::new(move |_| confirm_changes());

    view! {
        <section
            class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-3 sm:p-4"
            on:mousemove=on_mouse_move
            on:mouseup=stop_resize
            on:mouseleave=stop_resize
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
                search_columns=search_columns
                col_widths=col_widths
                sort_state=sort_state
                row_items=row_items
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
