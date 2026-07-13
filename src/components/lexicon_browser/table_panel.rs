use leptos::prelude::*;

use super::utils::{
    DATA_COLUMN_KEYS, input_to_option, option_to_input, parse_pipe_list,
};
use crate::structures::word_bank_entry::{PART_OF_SPEECH_OPTIONS, PartOfSpeech, UiLanguage, WordBankEntry};
use crate::utils::i18n::{field_label, tr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableActionKind {
    ConfirmChanges,
    AddEntries,
}

#[component]
pub fn LexiconTablePanel(
    lang: ReadSignal<UiLanguage>,
    is_query_mode: bool,
    search_columns: ReadSignal<Vec<bool>>,
    col_widths: ReadSignal<Vec<u16>>,
    sort_state: ReadSignal<Vec<(usize, bool)>>,
    row_items: Signal<Vec<(usize, WordBankEntry)>>,
    on_sort_by_column: Callback<(usize, bool)>,
    on_start_resize: Callback<(usize, leptos::ev::MouseEvent)>,
    on_begin_selected_drag: Callback<(usize, bool)>,
    on_drag_over_selected: Callback<(usize, leptos::ev::MouseEvent)>,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_row_undo: WriteSignal<Vec<Option<WordBankEntry>>>,
    baseline_entries: ReadSignal<Vec<WordBankEntry>>,
    set_baseline_entries: WriteSignal<Vec<WordBankEntry>>,
    row_undo: ReadSignal<Vec<Option<WordBankEntry>>>,
    set_status: WriteSignal<String>,
    confirm_error: ReadSignal<String>,
    confirm_success: ReadSignal<String>,
    delete_marks: ReadSignal<Vec<bool>>,
    on_begin_delete_drag: Callback<(usize, bool)>,
    on_drag_over_delete: Callback<(usize, leptos::ev::MouseEvent)>,
    on_select_visible_click: Callback<()>,
    on_unselect_visible_click: Callback<()>,
    on_confirm_changes: Callback<()>,
    #[prop(optional)] action_kind: Option<TableActionKind>,
    #[prop(optional)] show_select_buttons: Option<bool>,
) -> impl IntoView {
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();
    let action_kind = action_kind.unwrap_or(TableActionKind::ConfirmChanges);
    let show_select_buttons = show_select_buttons.unwrap_or(true);
    let _ = (baseline_entries, set_baseline_entries, row_undo);

    view! {
        <div class="max-h-[420px] overflow-auto pr-0 sm:pr-1">
                <table class="w-max min-w-full border-collapse text-sm table-fixed whitespace-nowrap">
                    <colgroup>
                        {move || {
                            let total_columns = if is_query_mode {
                                DATA_COLUMN_COUNT
                            } else {
                                DATA_COLUMN_COUNT + 1
                            };
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
                                let op_header = if is_query_mode {
                                    Vec::new()
                                } else {
                                    vec!["delete"]
                                };
                                let headers = DATA_COLUMN_KEYS
                                    .into_iter()
                                    .chain(op_header)
                                    .collect::<Vec<_>>();

                                let current_sort = sort_state.get();
                                headers
                                    .iter()
                                    .enumerate()
                                    .map(|(idx, title)| {
                                        let sortable = idx < DATA_COLUMN_COUNT;
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
                                                        on_sort_by_column.run((idx, ev.shift_key()));
                                                    }
                                                }
                                            >
                                                {format!(
                                                    "{}{}",
                                                    if *title == "delete" {
                                                        tr(lang.get(), "删除", "Delete").to_string()
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
                                                        on_start_resize.run((idx, ev));
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
                            each=move || row_items.get()
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
                                                        on_begin_selected_drag.run((idx, entry.selected));
                                                    }
                                                }
                                                on:mouseover=move |ev: leptos::ev::MouseEvent| {
                                                    on_drag_over_selected.run((idx, ev));
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
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(20).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_present_participle)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(21).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_past_participle)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(22).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_passive_infinitive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(23).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_passive_present)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(24).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.verb_passive_past)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(25).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_singular_definite_genitive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(26).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_plural_definite_genitive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(27).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_singular_indefinite_genitive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(28).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.noun_plural_indefinite_genitive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(29).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.adjective_feminine_form)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(30).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.pronoun_object)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(31).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.pronoun_reflexive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(32).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.pronoun_plural_subject)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(33).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.pronoun_plural_object)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(34).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.pronoun_plural_reflexive)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(35).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.determinative_feminine_form)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(36).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.determinative_neuter_form)}
                                            </td>
                                            <td
                                                class="border border-slate-800 p-2"
                                                class:hidden=move || !search_columns.get().get(37).copied().unwrap_or(false)
                                            >
                                                {option_to_input(&entry.determinative_plural_form)}
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
                                                    on_begin_selected_drag.run((idx, entry.selected));
                                                }
                                            }
                                            on:mouseover=move |ev: leptos::ev::MouseEvent| {
                                                on_drag_over_selected.run((idx, ev));
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
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(20).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_present_participle) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_present_participle = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(21).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_past_participle) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_past_participle = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(22).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_passive_infinitive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_passive_infinitive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(23).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_passive_present) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_passive_present = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(24).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.verb_passive_past) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.verb_passive_past = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(25).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_singular_definite_genitive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_singular_definite_genitive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(26).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_plural_definite_genitive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_plural_definite_genitive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(27).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_singular_indefinite_genitive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_singular_indefinite_genitive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(28).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.noun_plural_indefinite_genitive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.noun_plural_indefinite_genitive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(29).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.adjective_feminine_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.adjective_feminine_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(30).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.pronoun_object) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.pronoun_object = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(31).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.pronoun_reflexive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.pronoun_reflexive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(32).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.pronoun_plural_subject) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.pronoun_plural_subject = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(33).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.pronoun_plural_object) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.pronoun_plural_object = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(34).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.pronoun_plural_reflexive) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.pronoun_plural_reflexive = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(35).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.determinative_feminine_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.determinative_feminine_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(36).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.determinative_neuter_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.determinative_neuter_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td class="border border-slate-800 p-1" class:hidden=move || !search_columns.get().get(37).copied().unwrap_or(false)><input type="text" prop:value=option_to_input(&entry.determinative_plural_form) on:input=move |ev| { let value = event_target_value(&ev); set_entries.update(|list| { if let Some(item) = list.get_mut(idx) { set_row_undo.update(|undo| { if undo.len() <= idx { undo.resize(idx + 1, None); } undo[idx] = Some(item.clone()); }); item.determinative_plural_form = input_to_option(&value); } }); } class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"/></td>
                                        <td
                                            class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-red-900/20"
                                            on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                                if ev.button() == 0 {
                                                    ev.prevent_default();
                                                    let checked = delete_marks.get_untracked().get(idx).copied().unwrap_or(false);
                                                    on_begin_delete_drag.run((idx, checked));
                                                }
                                            }
                                            on:mouseover=move |ev: leptos::ev::MouseEvent| {
                                                on_drag_over_delete.run((idx, ev));
                                            }
                                        >
                                            <input
                                                type="checkbox"
                                                prop:checked=move || {
                                                    delete_marks.get().get(idx).copied().unwrap_or(false)
                                                }
                                                class="pointer-events-none"
                                            />
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
            <div class="mt-3 flex flex-col gap-3 sm:flex-row sm:items-center">
                {if show_select_buttons {
                    view! {
                        <div class="grid grid-cols-1 gap-2 sm:flex sm:items-center sm:gap-2">
                            <button
                                type="button"
                                on:click=move |_| on_select_visible_click.run(())
                                class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-100 hover:bg-slate-700 sm:w-auto"
                            >
                                {move || tr(lang.get(), "全选", "Select All")}
                            </button>
                            <button
                                type="button"
                                on:click=move |_| on_unselect_visible_click.run(())
                                class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-100 hover:bg-slate-700 sm:w-auto"
                            >
                                {move || tr(lang.get(), "全不选", "Unselect All")}
                            </button>
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <div class="hidden sm:block"></div> }.into_any()
                }}
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
                    on:click=move |_| on_confirm_changes.run(())
                    class="w-full rounded border border-emerald-800 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                >
                    {move || match action_kind {
                        TableActionKind::ConfirmChanges => {
                            tr(lang.get(), "确认修改", "Confirm Changes")
                        }
                        TableActionKind::AddEntries => tr(lang.get(), "添加多条", "Add Entries"),
                    }}
                </button>
            </div>
    }
}
