use leptos::prelude::*;
use std::cmp::Ordering;

use crate::lexicon::{PART_OF_SPEECH_OPTIONS, WordEntry, parse_pipe_list};

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordEntry>>,
    set_entries: WriteSignal<Vec<WordEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
) -> impl IntoView {
    let (col_widths, set_col_widths) = signal(vec![
        120_u16, 90, 140, 160, 180, 180, 150, 120, 120, 120, 140, 140, 130, 130, 160, 210, 210,
        160, 160, 210,
    ]);
    let (baseline_entries, set_baseline_entries) = signal(entries.get_untracked());
    let (row_undo, set_row_undo) = signal(vec![None::<WordEntry>; entries.get_untracked().len()]);
    let (resize_state, set_resize_state) = signal(None::<(usize, i32, u16)>);
    let (sort_state, set_sort_state) = signal(Vec::<(usize, bool)>::new());

    Effect::new(move |_| {
        let _ = data_version.get();
        let current = entries.get();
        set_baseline_entries.set(current.clone());
        set_row_undo.set(vec![None; current.len()]);
    });

    let row_items = move || {
        entries
            .get()
            .into_iter()
            .enumerate()
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
    };
    let sort_by_column = move |col_idx: usize, with_secondary: bool| {
        if col_idx >= 19 {
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

        let current_entries = entries.get_untracked();
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

        set_entries.set(sorted_entries);
        set_baseline_entries.set(sorted_baseline);
        set_row_undo.set(sorted_undo);
        set_sort_state.set(sort_rules.clone());
        set_status.set(format!("已排序：{}", format_sort_rules(&sort_rules)));
    };

    view! {
        <section
            class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4"
            on:mousemove=on_mouse_move
            on:mouseup=stop_resize
            on:mouseleave=stop_resize
        >
            <div class="max-h-[420px] overflow-auto pr-1">
                <table class="w-full border-collapse text-sm table-fixed">
                    <colgroup>
                        {move || (0..20)
                            .map(|idx| {
                                view! {
                                    <col style=format!(
                                        "width:{}px",
                                        col_widths.get().get(idx).copied().unwrap_or(140)
                                    ) />
                                }
                            })
                            .collect_view()}
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
                                    "操作",
                                ];

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
                            key=|(idx, entry)| format!("{}-{}", idx, entry.id)
                            children=move |(idx, entry)| {
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
                                        <td class="border border-slate-800 p-1 text-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=entry.selected
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    set_entries.update(|list| {
                                                        if let Some(item) = list.get_mut(idx) {
                                                            set_row_undo.update(|undo| {
                                                                if undo.len() <= idx {
                                                                    undo.resize(idx + 1, None);
                                                                }
                                                                undo[idx] = Some(item.clone());
                                                            });
                                                            item.selected = checked;
                                                        }
                                                    });
                                                }
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
                            }
                        />
                    </tbody>
                </table>
            </div>
        </section>
    }
}

fn compare_entries_by_rules(a: &WordEntry, b: &WordEntry, rules: &[(usize, bool)]) -> Ordering {
    for (col_idx, asc) in rules {
        let ordering = compare_entries_by_column(a, b, *col_idx, *asc);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.id.cmp(&b.id)
}

fn compare_entries_by_column(a: &WordEntry, b: &WordEntry, col_idx: usize, ascending: bool) -> Ordering {
    let ordering = match col_idx {
        0 => a.id.cmp(&b.id),
        1 => a.selected.cmp(&b.selected),
        2 => a.part_of_speech.cmp(&b.part_of_speech),
        3 => a.tags.join("|").cmp(&b.tags.join("|")),
        4 => a.english.join("|").cmp(&b.english.join("|")),
        5 => a.chinese.join("|").cmp(&b.chinese.join("|")),
        6 => a.base_form.cmp(&b.base_form),
        7 => opt_cmp(&a.past_tense, &b.past_tense),
        8 => opt_cmp(&a.imperative, &b.imperative),
        9 => opt_cmp(&a.plural, &b.plural),
        10 => opt_cmp(&a.singular_definite, &b.singular_definite),
        11 => opt_cmp(&a.plural_definite, &b.plural_definite),
        12 => opt_cmp(&a.neuter_form, &b.neuter_form),
        13 => opt_cmp(&a.plural_form, &b.plural_form),
        14 => opt_cmp(&a.adjective_comparative, &b.adjective_comparative),
        15 => opt_cmp(
            &a.adjective_superlative_indefinite,
            &b.adjective_superlative_indefinite,
        ),
        16 => opt_cmp(&a.adjective_superlative_definite, &b.adjective_superlative_definite),
        17 => opt_cmp(&a.adverb_comparative, &b.adverb_comparative),
        18 => opt_cmp(&a.adverb_superlative, &b.adverb_superlative),
        _ => Ordering::Equal,
    };

    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

fn opt_cmp(a: &Option<String>, b: &Option<String>) -> Ordering {
    a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
}

fn header_name(col_idx: usize) -> &'static str {
    match col_idx {
        0 => "id",
        1 => "selected",
        2 => "part_of_speech",
        3 => "tags",
        4 => "english",
        5 => "chinese",
        6 => "base_form",
        7 => "past_tense",
        8 => "imperative",
        9 => "plural",
        10 => "singular_definite",
        11 => "plural_definite",
        12 => "neuter_form",
        13 => "plural_form",
        14 => "adjective_comparative",
        15 => "adjective_superlative_indefinite",
        16 => "adjective_superlative_definite",
        17 => "adverb_comparative",
        18 => "adverb_superlative",
        _ => "unknown",
    }
}

fn format_sort_rules(rules: &[(usize, bool)]) -> String {
    if rules.is_empty() {
        return "未应用排序".to_string();
    }

    rules
        .iter()
        .enumerate()
        .map(|(idx, (col, asc))| {
            format!(
                "{}. {}{}",
                idx + 1,
                header_name(*col),
                if *asc { "↑" } else { "↓" }
            )
        })
        .collect::<Vec<_>>()
        .join("，")
}

fn option_to_input(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn input_to_option(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
