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

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">"词库浏览器"</h2>
            <p class="mb-3 text-sm text-slate-400">
                "以表格形式查看、直接修改并删除词条（类似 Excel）。"
            </p>

            <div class="mb-3 flex flex-wrap items-center gap-3 rounded-lg border border-slate-800 bg-slate-900/70 p-3 text-xs">
                <span class="text-slate-300">"列宽自定义(px):"</span>
                <div class="flex items-center gap-1">
                    <span class="text-slate-400">"id"</span>
                    <input
                        type="number"
                        min="80"
                        max="600"
                        prop:value=move || col_widths.get().first().copied().unwrap_or(140).to_string()
                        on:input=move |ev| {
                            if let Ok(width) = event_target_value(&ev).parse::<u16>() {
                                set_col_widths.update(|cols| {
                                    if cols.len() >= 1 {
                                        cols[0] = width.clamp(80, 600);
                                    }
                                });
                            }
                        }
                        class="w-20 rounded border border-slate-700 bg-slate-950 px-2 py-1 text-slate-100"
                    />
                </div>
                <div class="flex items-center gap-1">
                    <span class="text-slate-400">"中文"</span>
                    <input
                        type="number"
                        min="120"
                        max="800"
                        prop:value=move || col_widths.get().get(3).copied().unwrap_or(220).to_string()
                        on:input=move |ev| {
                            if let Ok(width) = event_target_value(&ev).parse::<u16>() {
                                set_col_widths.update(|cols| {
                                    if cols.len() >= 4 {
                                        cols[3] = width.clamp(120, 800);
                                    }
                                });
                            }
                        }
                        class="w-20 rounded border border-slate-700 bg-slate-950 px-2 py-1 text-slate-100"
                    />
                </div>
                <div class="flex items-center gap-1">
                    <span class="text-slate-400">"英文"</span>
                    <input
                        type="number"
                        min="120"
                        max="800"
                        prop:value=move || col_widths.get().get(4).copied().unwrap_or(220).to_string()
                        on:input=move |ev| {
                            if let Ok(width) = event_target_value(&ev).parse::<u16>() {
                                set_col_widths.update(|cols| {
                                    if cols.len() >= 5 {
                                        cols[4] = width.clamp(120, 800);
                                    }
                                });
                            }
                        }
                        class="w-20 rounded border border-slate-700 bg-slate-950 px-2 py-1 text-slate-100"
                    />
                </div>
            </div>

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
                            <th class="border border-slate-800 px-2 py-2">"id"</th>
                            <th class="border border-slate-800 px-2 py-2">"词性"</th>
                            <th class="border border-slate-800 px-2 py-2">"原型"</th>
                            <th class="border border-slate-800 px-2 py-2">"中文（|）"</th>
                            <th class="border border-slate-800 px-2 py-2">"英文（|）"</th>
                            <th class="border border-slate-800 px-2 py-2">"tags（|）"</th>
                            <th class="border border-slate-800 px-2 py-2">"操作（恢复/撤销/删除）"</th>
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
