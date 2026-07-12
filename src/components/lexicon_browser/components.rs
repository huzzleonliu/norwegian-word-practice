use leptos::prelude::*;

use crate::app_state::WordBankState;
use super::structures::LexiconBrowserMode;
use super::utils::{
    compare_entries_by_rules, entry_matches_filter, format_sort_rules, header_name,
    input_to_option, option_to_input, parse_pipe_list,
};
use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech, WordBankEntry};
use crate::utils::i18n::{field_label, tr};
use crate::utils::dictionary::validate_existing_entry;

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordBankEntry>>,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
    mode: LexiconBrowserMode,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    const DATA_COLUMN_COUNT: usize = 20;
    let is_query_mode = mode == LexiconBrowserMode::Query;
    let (col_widths, set_col_widths) = signal(vec![
        120_u16, 90, 140, 160, 180, 180, 150, 120, 120, 120, 140, 140, 130, 130, 160, 210, 210,
        160, 160, 160, 210,
    ]);
    let (baseline_entries, set_baseline_entries) = signal(entries.get_untracked());
    let (draft_entries, set_draft_entries) = signal(entries.get_untracked());
    let (row_undo, set_row_undo) =
        signal(vec![None::<WordBankEntry>; entries.get_untracked().len()]);
    let (resize_state, set_resize_state) = signal(None::<(usize, i32, u16)>);
    let (selection_drag_target, set_selection_drag_target) = signal(None::<bool>);
    let (sort_state, set_sort_state) = signal(Vec::<(usize, bool)>::new());
    let (search_text, set_search_text) = signal(String::new());
    let (search_query, set_search_query) = signal(String::new());
    let (search_columns, set_search_columns) = signal({
        let mut cols = vec![true; DATA_COLUMN_COUNT];
        if let Some(id_col) = cols.get_mut(0) {
            *id_col = false;
        }
        cols
    });
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

    let row_items = move || {
        let query = search_query.get();
        let columns = search_columns.get();
        draft_entries
            .get()
            .into_iter()
            .enumerate()
            .filter(|(_, entry)| entry_matches_filter(entry, &query, &columns))
            .collect::<Vec<(usize, WordBankEntry)>>()
    };
    let min_width_for = |idx: usize| -> u16 {
        match idx {
            1 => 70,
            7..=19 => 100,
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
    let apply_search = move |_| {
        let query = search_text.get().trim().to_string();
        let columns = search_columns.get_untracked();
        set_search_query.set(query.clone());

        if query.is_empty() {
            set_status.set(format!(
                "{} {} {}",
                tr(lang.get_untracked(), "已显示全部条目，共", "Showing all entries,"),
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
            tr(lang.get_untracked(), "查找完成：匹配", "Search complete: matched"),
            count,
            tr(lang.get_untracked(), "条。", "entries.")
        ));
    };
    let confirm_changes = move |_| {
        set_confirm_error.set(String::new());
        set_confirm_success.set(String::new());

        let current_entries = draft_entries.get_untracked();
        let mut entries_to_commit = current_entries.clone();

        if !is_query_mode {
            let mut errors = Vec::new();
            let mut validated_entries = Vec::with_capacity(current_entries.len());
            for (idx, item) in current_entries.iter().enumerate() {
                match validate_existing_entry(item, &current_entries, idx) {
                    Ok(validated_entry) => validated_entries.push(validated_entry),
                    Err(err) => errors.push(format!("第 {} 行（id: {}）{}", idx + 1, item.id, err)),
                }
            }

            if !errors.is_empty() {
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

            entries_to_commit = validated_entries;
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
    let select_visible_click = move |_| set_visible_selected(true);
    let unselect_visible_click = move |_| set_visible_selected(false);

    view! {
        <section
            class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4"
            on:mousemove=on_mouse_move
            on:mouseup=stop_resize
            on:mouseleave=stop_resize
        >
            <div class="mb-4 rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                <div class="flex flex-wrap items-center gap-2">
                    <input
                        type="text"
                        placeholder=move || tr(lang.get(), "输入要查找的字符", "Type to search")
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        class="min-w-60 flex-1 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
                    />
                    <button
                        type="button"
                        on:click=apply_search
                        class="rounded border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "查找", "Search")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![true; DATA_COLUMN_COUNT])
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "全选列", "Select all columns")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![false; DATA_COLUMN_COUNT])
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "全不选列", "Unselect all columns")}
                    </button>
                </div>
                <div class="mt-3">
                    <button
                        type="button"
                        on:click=move |_| {
                            set_search_scope_expanded.update(|expanded| *expanded = !*expanded)
                        }
                        class="inline-flex items-center gap-2 rounded border border-slate-700 bg-slate-900 px-3 py-1 text-xs font-medium text-slate-200 hover:bg-slate-800"
                    >
                        {move || {
                            if search_scope_expanded.get() {
                                tr(lang.get(), "隐藏搜索列范围", "Hide Search Columns")
                            } else {
                                tr(lang.get(), "展开搜索列范围", "Show Search Columns")
                            }
                        }}
                    </button>
                    {move || {
                        if search_scope_expanded.get() {
                            view! {
                                <div class="mt-3 space-y-3">
                                    {[
                                        ("core", vec![0, 1, 2, 3, 4, 5, 6]),
                                        ("verb", vec![7, 8, 9]),
                                        ("noun", vec![10, 11, 12]),
                                        ("adjective", vec![13, 14, 15, 16, 17]),
                                        ("adverb", vec![18, 19]),
                                    ]
                                        .into_iter()
                                        .map(|(group_name, indices)| {
                                            view! {
                                                <section class="rounded border border-slate-800 bg-slate-950/40 p-2">
                                                    <p class="mb-2 text-xs font-semibold text-slate-400">
                                                        {move || match group_name {
                                                            "core" => tr(lang.get(), "基础组", "Core"),
                                                            "verb" => tr(lang.get(), "动词变体组", "Verb Forms"),
                                                            "noun" => tr(lang.get(), "名词变体组", "Noun Forms"),
                                                            "adjective" => tr(lang.get(), "形容词变体组", "Adjective Forms"),
                                                            "adverb" => tr(lang.get(), "副词变体组", "Adverb Forms"),
                                                            _ => group_name,
                                                        }}
                                                    </p>
                                                    <div class="grid grid-cols-2 gap-2 md:grid-cols-4 lg:grid-cols-5">
                                                        {indices
                                                            .into_iter()
                                                            .map(|idx| {
                                                                view! {
                                                                    <label class="inline-flex items-center gap-2 rounded border border-slate-800 bg-slate-950/60 px-2 py-1 text-xs text-slate-300">
                                                                        <input
                                                                            type="checkbox"
                                                                            prop:checked=move || {
                                                                                search_columns.get().get(idx).copied().unwrap_or(false)
                                                                            }
                                                                            on:change=move |ev| {
                                                                                let checked = event_target_checked(&ev);
                                                                                set_search_columns.update(|cols| {
                                                                                    if idx < cols.len() {
                                                                                        cols[idx] = checked;
                                                                                    }
                                                                                });
                                                                            }
                                                                        />
                                                                        <span>{move || field_label(lang.get(), header_name(idx))}</span>
                                                                    </label>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                </section>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <></> }.into_any()
                        }
                    }}
                </div>
            </div>
            <div class="max-h-[420px] overflow-auto pr-1">
                <table class="w-full border-collapse text-sm table-fixed">
                    <colgroup>
                        {move || {
                            let total_columns = if is_query_mode { DATA_COLUMN_COUNT } else { 21 };
                            (0..total_columns)
                                .map(|idx| {
                                    view! {
                                        <col
                                            class:hidden=move || {
                                                idx < DATA_COLUMN_COUNT
                                                    && !search_columns
                                                        .get()
                                                        .get(idx)
                                                        .copied()
                                                        .unwrap_or(false)
                                            }
                                            style=format!(
                                                "width:{}px",
                                                col_widths.get().get(idx).copied().unwrap_or(140)
                                            )
                                        />
                                    }
                                })
                                .collect_view()
                        }}
                    </colgroup>
                    <thead>
                        <tr class="sticky top-0 z-10 bg-slate-900/95 text-left text-slate-300">
                            {move || {
                                let headers = [
                                    "id",
                                    "selected",
                                    "part_of_speech",
                                    "tags",
                                    "english",
                                    "chinese",
                                    "base_form",
                                    "verb_present_tense",
                                    "verb_past_tense",
                                    "verb_imperative",
                                    "noun_plural",
                                    "noun_singular_definite",
                                    "noun_plural_definite",
                                    "adjective_neuter_form",
                                    "adjective_plural_form",
                                    "adjective_comparative",
                                    "adjective_superlative_indefinite",
                                    "adjective_superlative_definite",
                                    "adverb_comparative",
                                    "adverb_superlative",
                                ];
                                let op_header = if is_query_mode {
                                    Vec::new()
                                } else {
                                    vec!["operation"]
                                };
                                let headers = headers
                                    .into_iter()
                                    .chain(op_header)
                                    .collect::<Vec<_>>();

                                let current_sort = sort_state.get();
                                headers
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, title)| {
                                        let sortable = idx < 20;
                                        let indicator = current_sort
                                            .iter()
                                            .enumerate()
                                            .find_map(|(order_idx, (sort_idx, asc))| {
                                                if *sort_idx == idx {
                                                    Some(format!(
                                                        " {}{}",
                                                        order_idx + 1,
                                                        if *asc { "↑" } else { "↓" }
                                                    ))
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or_default();
                                        view! {
                                            <th
                                                class="relative border border-slate-800 px-2 py-2"
                                                class:cursor-pointer=sortable
                                                class:hidden=move || {
                                                    idx < DATA_COLUMN_COUNT
                                                        && !search_columns
                                                            .get()
                                                            .get(idx)
                                                            .copied()
                                                            .unwrap_or(false)
                                                }
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    if sortable {
                                                        sort_by_column(idx, ev.shift_key());
                                                    }
                                                }
                                            >
                                                {format!(
                                                    "{}{}",
                                                    if *title == "operation" {
                                                        tr(lang.get(), "操作", "Actions").to_string()
                                                    } else if *title == "tags"
                                                        || *title == "english"
                                                        || *title == "chinese"
                                                    {
                                                        format!("{}(|)", field_label(lang.get(), title))
                                                    } else {
                                                        field_label(lang.get(), title).to_string()
                                                    },
                                                    indicator
                                                )}
                                                <div
                                                    class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80"
                                                    on:mousedown=move |ev| {
                                                        ev.stop_propagation();
                                                        start_resize(idx, ev);
                                                    }
                                                    on:click=move |ev| ev.stop_propagation()
                                                ></div>
                                            </th>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=row_items
                            key=|(idx, entry)| format!("{}-{}-{}", idx, entry.id, entry.selected)
                            children=move |(idx, entry)| {
                                if is_query_mode {
                                    view! {
                                        <tr class="align-top">
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(0).copied().unwrap_or(false)
                                            >
                                                {entry.id.clone()}
                                            </td>
                                            <td
                                                class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
                                                class:hidden=move || !search_columns.get().get(1).copied().unwrap_or(false)
                                                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                                    if ev.button() == 0 {
                                                        ev.prevent_default();
                                                        begin_selected_drag(idx, entry.selected);
                                                    }
                                                }
                                                on:mouseover=move |ev: leptos::ev::MouseEvent| {
                                                    drag_over_selected(idx, ev);
                                                }
                                            >
                                                <input
                                                    type="checkbox"
                                                    prop:checked=entry.selected
                                                    class="pointer-events-none"
                                                />
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(2).copied().unwrap_or(false)
                                            >
                                                {move || entry.part_of_speech.display_name(lang.get()).to_string()}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(3).copied().unwrap_or(false)
                                            >
                                                {entry.tags.join(" | ")}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(4).copied().unwrap_or(false)
                                            >
                                                {entry.english.join(" | ")}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(5).copied().unwrap_or(false)
                                            >
                                                {entry.chinese.join(" | ")}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(6).copied().unwrap_or(false)
                                            >
                                                {entry.base_form.clone()}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(7).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_present_tense)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(8).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_past_tense)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(9).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_imperative)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(10).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_plural)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(11).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_singular_definite)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(12).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_plural_definite)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(13).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_neuter_form)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(14).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_plural_form)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(15).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_comparative)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(16).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_superlative_indefinite)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(17).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_superlative_definite)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(18).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adverb_comparative)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(19).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adverb_superlative)}
                                            </td>
                                        </tr>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <tr class="align-top">
                                        <td
                                            class="border border-slate-800 p-1"
                                            class:hidden=move || !search_columns.get().get(0).copied().unwrap_or(false)
                                        >
                                            <span class="block rounded border border-slate-800 bg-slate-950 px-2 py-1 text-xs text-slate-300">
                                                {entry.id.clone()}
                                            </span>
                                        </td>
                                        <td
                                            class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
                                            class:hidden=move || !search_columns.get().get(1).copied().unwrap_or(false)
                                            on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                                if ev.button() == 0 {
                                                    ev.prevent_default();
                                                    begin_selected_drag(idx, entry.selected);
                                                }
                                            }
                                            on:mouseover=move |ev: leptos::ev::MouseEvent| {
                                                drag_over_selected(idx, ev);
                                            }
                                        >
                                            <input
                                                type="checkbox"
                                                prop:checked=entry.selected
                                                class="pointer-events-none"
                                            />
                                        </td>
                                        <td
                                            class="border border-slate-800 p-1"
                                            class:hidden=move || !search_columns.get().get(2).copied().unwrap_or(false)
                                        >
                                            <select
                                                prop:value=entry.part_of_speech.as_key().to_string()
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    let parsed = match PartOfSpeech::from_key(&value) {
                                                        Ok(parsed) => parsed,
                                                        Err(err) => {
                                                            set_status.set(format!(
                                                                "{} {err}",
                                                                tr(lang.get_untracked(), "词性更新失败：", "Failed to update part of speech:")
                                                            ));
                                                            return;
                                                        }
                                                    };
                                                    set_entries.update(|list| {
                                                        if let Some(item) = list.get_mut(idx) {
                                                            set_row_undo.update(|undo| {
                                                                if undo.len() <= idx {
                                                                    undo.resize(idx + 1, None);
                                                                }
                                                                undo[idx] = Some(item.clone());
                                                            });
                                                            item.part_of_speech = parsed;
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            >
                                                {PART_OF_SPEECH_OPTIONS
                                                    .iter()
                                                    .map(|option| {
                                                        view! {
                                                            <option value=*option>
                                                                {move || {
                                                                    PartOfSpeech::from_key(option)
                                                                        .map(|pos| pos.display_name(lang.get()).to_string())
                                                                        .unwrap_or_else(|_| (*option).to_string())
                                                                }}
                                                            </option>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </select>
                                        </td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(3).copied().unwrap_or(false)><input type="text" prop:value=entry.tags.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.tags = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(4).copied().unwrap_or(false)><input type="text" prop:value=entry.english.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.english = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(5).copied().unwrap_or(false)><input type="text" prop:value=entry.chinese.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.chinese = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(6).copied().unwrap_or(false)><input type="text" prop:value=entry.base_form.clone() on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.base_form = value; } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(7).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_present_tense) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_present_tense = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(8).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_past_tense) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_past_tense = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(9).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_imperative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_imperative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(10).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_plural) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_plural = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(11).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_singular_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_singular_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(12).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_plural_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_plural_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(13).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_neuter_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_neuter_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(14).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_plural_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_plural_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(15).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_comparative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_comparative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(16).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_superlative_indefinite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_superlative_indefinite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(17).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_superlative_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_superlative_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(18).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adverb_comparative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adverb_comparative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(19).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adverb_superlative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adverb_superlative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1">
                                            <div class="flex flex-wrap gap-1">
                                                <button type="button" on:click=move |_| {
                                                    if let Some(original) = baseline_entries.get_untracked().get(idx).cloned() {
                                                        set_entries.update(|list| {
                                                            if let Some(item) = list.get_mut(idx) {
                                                                *item = original;
                                                            }
                                                        });
                                                        set_status.set(
                                                            tr(lang.get_untracked(), "已恢复该行初始值。", "Row restored to initial values.")
                                                                .to_string(),
                                                        );
                                                    }
                                                } class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700">
                                                    {move || tr(lang.get(), "恢复原值", "Reset")}
                                                </button>
                                                <button type="button" on:click=move |_| {
                                                    let undo_value = row_undo.get_untracked().get(idx).cloned().flatten();
                                                    if let Some(previous) = undo_value {
                                                        set_entries.update(|list| {
                                                            if let Some(item) = list.get_mut(idx) {
                                                                *item = previous;
                                                            }
                                                        });
                                                        set_row_undo.update(|undo| {
                                                            if idx < undo.len() {
                                                                undo[idx] = None;
                                                            }
                                                        });
                                                        set_status.set(
                                                            tr(lang.get_untracked(), "已撤销该行最近一次修改。", "Reverted latest row change.")
                                                                .to_string(),
                                                        );
                                                    }
                                                } class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700">
                                                    {move || tr(lang.get(), "撤销", "Undo")}
                                                </button>
                                                <button type="button" on:click=move |_| {
                                                    set_entries.update(|list| {
                                                        if idx < list.len() {
                                                            list.remove(idx);
                                                        }
                                                    });
                                                    set_baseline_entries.update(|base| {
                                                        if idx < base.len() {
                                                            base.remove(idx);
                                                        }
                                                    });
                                                    set_row_undo.update(|undo| {
                                                        if idx < undo.len() {
                                                            undo.remove(idx);
                                                        }
                                                    });
                                                    set_status.set(
                                                        tr(lang.get_untracked(), "已删除 1 条词条。", "Deleted 1 entry.").to_string(),
                                                    );
                                                } class="rounded border border-red-800 bg-red-900/80 px-2 py-1 text-xs text-red-100 hover:bg-red-800">
                                                    {move || tr(lang.get(), "删除", "Delete")}
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                                    .into_any()
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
            <div class="mt-3 flex items-center gap-3">
                <div class="flex items-center gap-2">
                    <button
                        type="button"
                        on:click=select_visible_click
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "全选", "Select All")}
                    </button>
                    <button
                        type="button"
                        on:click=unselect_visible_click
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-100 hover:bg-slate-700"
                    >
                        {move || tr(lang.get(), "全不选", "Unselect All")}
                    </button>
                </div>
                <div class="min-h-6 flex-1 text-sm">
                    {move || {
                        if !confirm_error.get().is_empty() {
                            view! { <p class="text-red-400">{confirm_error.get()}</p> }.into_any()
                        } else if !confirm_success.get().is_empty() {
                            view! { <p class="text-emerald-400">{confirm_success.get()}</p> }.into_any()
                        } else {
                            view! { <p class="text-slate-500">""</p> }.into_any()
                        }
                    }}
                </div>
                <button
                    type="button"
                    on:click=confirm_changes
                    class="rounded border border-emerald-800 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                >
                    {move || tr(lang.get(), "确认修改", "Confirm Changes")}
                </button>
            </div>
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
