use leptos::prelude::*;

use super::structures::{LexiconBrowserMode, PART_OF_SPEECH_OPTIONS, WordEntry};
use super::utils::{
    compare_entries_by_rules, entry_matches_filter, format_sort_rules, header_name,
    input_to_option, option_to_input, parse_pipe_list,
};
use crate::utils::dictionary::validate_existing_entry;

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordEntry>>,
    set_entries: WriteSignal<Vec<WordEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
    mode: LexiconBrowserMode,
) -> impl IntoView {
    const DATA_COLUMN_COUNT: usize = 19;
    let is_query_mode = mode == LexiconBrowserMode::Query;
    let (col_widths, set_col_widths) = signal(vec![
        120_u16, 90, 140, 160, 180, 180, 150, 120, 120, 120, 140, 140, 130, 130, 160, 210, 210,
        160, 160, 210,
    ]);
    let (baseline_entries, set_baseline_entries) = signal(entries.get_untracked());
    let (draft_entries, set_draft_entries) = signal(entries.get_untracked());
    let (row_undo, set_row_undo) = signal(vec![None::<WordEntry>; entries.get_untracked().len()]);
    let (resize_state, set_resize_state) = signal(None::<(usize, i32, u16)>);
    let (selection_drag_target, set_selection_drag_target) = signal(None::<bool>);
    let (sort_state, set_sort_state) = signal(Vec::<(usize, bool)>::new());
    let (search_text, set_search_text) = signal(String::new());
    let (search_query, set_search_query) = signal(String::new());
    let (search_columns, set_search_columns) = signal(vec![true; DATA_COLUMN_COUNT]);
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
            .collect::<Vec<(usize, WordEntry)>>()
    };
    let min_width_for = |idx: usize| -> u16 {
        match idx {
            1 => 70,
            7..=18 => 100,
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
        set_status.set(format!("已排序：{}", format_sort_rules(&sort_rules)));
    };
    let apply_search = move |_| {
        let query = search_text.get().trim().to_string();
        let columns = search_columns.get_untracked();
        set_search_query.set(query.clone());

        if query.is_empty() {
            set_status.set(format!(
                "已显示全部条目，共 {} 条。",
                draft_entries.get_untracked().len()
            ));
            return;
        }

        let count = draft_entries
            .get_untracked()
            .iter()
            .filter(|entry| entry_matches_filter(entry, &query, &columns))
            .count();
        set_status.set(format!("查找完成：匹配 {} 条。", count));
    };
    let confirm_changes = move |_| {
        set_confirm_error.set(String::new());
        set_confirm_success.set(String::new());

        let current_entries = draft_entries.get_untracked();

        let mut errors = Vec::new();
        for (idx, item) in current_entries.iter().enumerate() {
            if let Err(err) = validate_existing_entry(item, &current_entries, idx) {
                errors.push(format!("第 {} 行（id: {}）{}", idx + 1, item.id, err));
            }
        }

        if !errors.is_empty() {
            let message = errors.join("；");
            set_confirm_error.set(message.clone());
            set_status.set("确认失败：存在不合法修改。".to_string());
            return;
        }

        set_committed_entries.set(current_entries.clone());
        set_draft_entries.set(current_entries.clone());
        set_baseline_entries.set(current_entries.clone());
        set_row_undo.set(vec![None; current_entries.len()]);
        set_confirm_success.set("修改成功".to_string());
        set_status.set("词库修改已确认并应用。".to_string());
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
            set_status.set("当前没有可操作的显示条目。".to_string());
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
            "已将当前显示的 {} 条词条设置为 {}。",
            visible_indices.len(),
            if checked { "选中" } else { "不选中" }
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
                        placeholder="输入要查找的字符"
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        class="min-w-60 flex-1 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm"
                    />
                    <button
                        type="button"
                        on:click=apply_search
                        class="rounded border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700"
                    >
                        "查找"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![true; DATA_COLUMN_COUNT])
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700"
                    >
                        "全选列"
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![false; DATA_COLUMN_COUNT])
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700"
                    >
                        "全不选列"
                    </button>
                </div>
                <div class="mt-3 grid grid-cols-2 gap-2 md:grid-cols-4 lg:grid-cols-5">
                    {(0..DATA_COLUMN_COUNT)
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
                                    <span>{header_name(idx)}</span>
                                </label>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <div class="max-h-[420px] overflow-auto pr-1">
                <table class="w-full border-collapse text-sm table-fixed">
                    <colgroup>
                        {move || {
                            let total_columns = if is_query_mode { DATA_COLUMN_COUNT } else { 20 };
                            (0..total_columns)
                                .map(|idx| {
                                    view! {
                                        <col style=format!(
                                            "width:{}px",
                                            col_widths.get().get(idx).copied().unwrap_or(140)
                                        ) />
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
                                    "tags(|)",
                                    "english(|)",
                                    "chinese(|)",
                                    "base_form",
                                    "past_tense",
                                    "imperative",
                                    "plural",
                                    "singular_definite",
                                    "plural_definite",
                                    "neuter_form",
                                    "plural_form",
                                    "adjective_comparative",
                                    "adjective_superlative_indefinite",
                                    "adjective_superlative_definite",
                                    "adverb_comparative",
                                    "adverb_superlative",
                                ];
                                let op_header = if is_query_mode {
                                    Vec::new()
                                } else {
                                    vec!["操作"]
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
                                        let sortable = idx < 19;
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
                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                    if sortable {
                                                        sort_by_column(idx, ev.shift_key());
                                                    }
                                                }
                                            >
                                                {format!("{title}{indicator}")}
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
                                            <td class="border border-slate-800 p-2">{entry.id.clone()}</td>
                                            <td
                                                class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
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
                                            <td class="border border-slate-800 p-2">{entry.part_of_speech.clone()}</td>
                                            <td class="border border-slate-800 p-2">{entry.tags.join(" | ")}</td>
                                            <td class="border border-slate-800 p-2">{entry.english.join(" | ")}</td>
                                            <td class="border border-slate-800 p-2">{entry.chinese.join(" | ")}</td>
                                            <td class="border border-slate-800 p-2">{entry.base_form.clone()}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.past_tense)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.imperative)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.plural)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.singular_definite)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.plural_definite)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.neuter_form)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.plural_form)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.adjective_comparative)}</td>
                                            <td class="border border-slate-800 p-2">
                                                {option_to_input(&entry.adjective_superlative_indefinite)}
                                            </td>
                                            <td class="border border-slate-800 p-2">
                                                {option_to_input(&entry.adjective_superlative_definite)}
                                            </td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.adverb_comparative)}</td>
                                            <td class="border border-slate-800 p-2">{option_to_input(&entry.adverb_superlative)}</td>
                                        </tr>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <tr class="align-top">
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.id.clone()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    set_entries.update(|list| {
                                                        if let Some(item) = list.get_mut(idx) {
                                                            set_row_undo.update(|undo| {
                                                                if undo.len() <= idx {
                                                                    undo.resize(idx + 1, None);
                                                                }
                                                                undo[idx] = Some(item.clone());
                                                            });
                                                            item.id = value;
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td
                                            class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
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
                                        <td class="border border-slate-800 p-1">
                                            <select
                                                prop:value=entry.part_of_speech.clone()
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    set_entries.update(|list| {
                                                        if let Some(item) = list.get_mut(idx) {
                                                            set_row_undo.update(|undo| {
                                                                if undo.len() <= idx {
                                                                    undo.resize(idx + 1, None);
                                                                }
                                                                undo[idx] = Some(item.clone());
                                                            });
                                                            item.part_of_speech = value;
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            >
                                                {PART_OF_SPEECH_OPTIONS
                                                    .iter()
                                                    .map(|option| {
                                                        view! { <option value=*option>{*option}</option> }
                                                    })
                                                    .collect_view()}
                                            </select>
                                        </td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=entry.tags.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.tags = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=entry.english.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.english = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=entry.chinese.join(" | ") on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.chinese = parse_pipe_list(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=entry.base_form.clone() on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.base_form = value; } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.past_tense) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.past_tense = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.imperative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.imperative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.plural) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.plural = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.singular_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.singular_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.plural_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.plural_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.neuter_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.neuter_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.plural_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.plural_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.adjective_comparative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_comparative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.adjective_superlative_indefinite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_superlative_indefinite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.adjective_superlative_definite) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_superlative_definite = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.adverb_comparative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adverb_comparative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1"><input type="text" prop:value=option_to_input(&entry.adverb_superlative) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adverb_superlative = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1">
                                            <div class="flex flex-wrap gap-1">
                                                <button type="button" on:click=move |_| {
                                                    if let Some(original) = baseline_entries.get_untracked().get(idx).cloned() {
                                                        set_entries.update(|list| {
                                                            if let Some(item) = list.get_mut(idx) {
                                                                *item = original;
                                                            }
                                                        });
                                                        set_status.set("已恢复该行初始值。".to_string());
                                                    }
                                                } class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700">"恢复原值"</button>
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
                                                        set_status.set("已撤销该行最近一次修改。".to_string());
                                                    }
                                                } class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700">"撤销"</button>
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
                                                    set_status.set("已删除 1 条词条。".to_string());
                                                } class="rounded border border-red-800 bg-red-900/80 px-2 py-1 text-xs text-red-100 hover:bg-red-800">"删除"</button>
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
                        "全选"
                    </button>
                    <button
                        type="button"
                        on:click=unselect_visible_click
                        class="rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-100 hover:bg-slate-700"
                    >
                        "全不选"
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
                    "确认修改"
                </button>
            </div>
        </section>
    }
}

fn update_selected_cell(
    idx: usize,
    checked: bool,
    set_entries: WriteSignal<Vec<WordEntry>>,
    set_row_undo: WriteSignal<Vec<Option<WordEntry>>>,
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
