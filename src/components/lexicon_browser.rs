use leptos::prelude::*;

use crate::lexicon::{WordEntry, parse_pipe_list};

#[component]
pub fn LexiconBrowser(
    entries: ReadSignal<Vec<WordEntry>>,
    set_entries: WriteSignal<Vec<WordEntry>>,
    set_status: WriteSignal<String>,
    data_version: ReadSignal<u64>,
) -> impl IntoView {
    let (col_widths, set_col_widths) = signal(vec![140_u16, 110, 150, 220, 220, 180, 220]);
    let (baseline_entries, set_baseline_entries) = signal(entries.get_untracked());
    let (row_undo, set_row_undo) = signal(vec![None::<WordEntry>; entries.get_untracked().len()]);
    let (resize_state, set_resize_state) = signal(None::<(usize, i32, u16)>);

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
            0 => 80,
            1 => 80,
            2 => 100,
            3 => 120,
            4 => 120,
            5 => 120,
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
                        <col style=move || format!("width:{}px", col_widths.get().first().copied().unwrap_or(140)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(1).copied().unwrap_or(110)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(2).copied().unwrap_or(150)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(3).copied().unwrap_or(220)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(4).copied().unwrap_or(220)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(5).copied().unwrap_or(180)) />
                        <col style=move || format!("width:{}px", col_widths.get().get(6).copied().unwrap_or(220)) />
                    </colgroup>
                    <thead>
                        <tr class="sticky top-0 z-10 bg-slate-900/95 text-left text-slate-300">
                            <th class="relative border border-slate-800 px-2 py-2">
                                "id"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(0, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "词性"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(1, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "原型"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(2, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "中文（|）"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(3, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "英文（|）"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(4, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "tags（|）"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(5, ev)></div>
                            </th>
                            <th class="relative border border-slate-800 px-2 py-2">
                                "操作（恢复/撤销/删除）"
                                <div class="absolute right-0 top-0 h-full w-1 cursor-col-resize bg-slate-700/0 hover:bg-slate-500/80" on:mousedown=move |ev| start_resize(6, ev)></div>
                            </th>
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
                                                            item.id = value.clone();
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.part_of_speech.clone()
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
                                                            item.part_of_speech = value.clone();
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.norwegian_base.clone()
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
                                                            item.norwegian_base = value.clone();
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.chinese.join(" | ")
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
                                                            item.chinese = parse_pipe_list(&value);
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.english.join(" | ")
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
                                                            item.english = parse_pipe_list(&value);
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <input
                                                type="text"
                                                prop:value=entry.tags.join(" | ")
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
                                                            item.tags = parse_pipe_list(&value);
                                                        }
                                                    });
                                                }
                                                class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                                            />
                                        </td>
                                        <td class="border border-slate-800 p-1">
                                            <div class="flex flex-wrap gap-1">
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
                                                        if let Some(original) = baseline_entries.get_untracked().get(idx).cloned() {
                                                            set_entries.update(|list| {
                                                                if let Some(item) = list.get_mut(idx) {
                                                                    *item = original;
                                                                }
                                                            });
                                                            set_status.set("已恢复该行初始值。".to_string());
                                                        }
                                                    }
                                                    class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700"
                                                >
                                                    "恢复原值"
                                                </button>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
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
                                                    }
                                                    class="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-100 hover:bg-slate-700"
                                                >
                                                    "撤销"
                                                </button>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
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
                                                    }
                                                    class="rounded border border-red-800 bg-red-900/80 px-2 py-1 text-xs text-red-100 hover:bg-red-800"
                                                >
                                                    "删除"
                                                </button>
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
