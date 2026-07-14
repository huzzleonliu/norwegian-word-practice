//! 词库浏览器表格面板：渲染行列、编辑控件、拖拽选择与底部动作区。

use leptos::prelude::*;

use super::utils::{DATA_COLUMN_KEYS, input_to_option, parse_pipe_list};
use crate::structures::field_meta::{entry_field_value, field_meta};
use crate::structures::word_bank_entry::{
    PART_OF_SPEECH_OPTIONS, PartOfSpeech, UiLanguage, WordBankEntry,
};
use crate::utils::i18n::{field_label, tr};

/// 表格底部主操作按钮类型：编辑模式确认修改，批量新增模式提交新增。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableActionKind {
    ConfirmChanges,
    AddEntries,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColumnEditorKind {
    ReadOnly,
    SelectedToggle,
    PartOfSpeechSelect,
    PipeListText,
    RequiredText,
    OptionalText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ColumnConfig {
    key: &'static str,
    editor_kind: ColumnEditorKind,
    hash_field: bool,
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
    #[prop(optional)] on_mark_hash_dirty: Option<Callback<String>>,
    #[prop(optional)] action_kind: Option<TableActionKind>,
    #[prop(optional)] show_select_buttons: Option<bool>,
) -> impl IntoView {
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();
    let action_kind = action_kind.unwrap_or(TableActionKind::ConfirmChanges);
    let show_select_buttons = show_select_buttons.unwrap_or(true);
    let on_mark_hash_dirty = on_mark_hash_dirty.unwrap_or(Callback::new(|_| {}));
    let column_configs = build_column_configs();
    let header_column_configs = column_configs.clone();
    let row_column_configs = column_configs.clone();
    let _ = (baseline_entries, set_baseline_entries, row_undo);

    view! {
        <div class="max-h-[420px] overflow-auto pr-0 sm:pr-1">
            <table class="w-max min-w-full border-collapse table-fixed whitespace-nowrap text-sm">
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
                            let current_sort = sort_state.get();
                            let mut headers = header_column_configs
                                .iter()
                                .map(|config| config.key)
                                .collect::<Vec<_>>();
                            if !is_query_mode {
                                headers.push("delete");
                            }

                            headers
                                .iter()
                                .enumerate()
                                .map(|(idx, key)| {
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
                                    let title = column_header_text(lang.get(), key);
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
                                            {format!("{title}{indicator}")}
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
                            let data_cells = row_column_configs
                                .iter()
                                .enumerate()
                                .map(|(col_idx, config)| {
                                    let config = *config;
                                    if is_query_mode {
                                        render_query_cell(
                                            idx,
                                            entry.clone(),
                                            col_idx,
                                            config,
                                            lang,
                                            search_columns,
                                            on_begin_selected_drag,
                                            on_drag_over_selected,
                                        )
                                    } else {
                                        render_edit_cell(
                                            idx,
                                            entry.clone(),
                                            col_idx,
                                            config,
                                            lang,
                                            search_columns,
                                            on_begin_selected_drag,
                                            on_drag_over_selected,
                                            set_entries,
                                            set_row_undo,
                                            set_status,
                                            on_mark_hash_dirty,
                                        )
                                    }
                                })
                                .collect_view();

                            let delete_cell = if is_query_mode {
                                view! { <></> }.into_any()
                            } else {
                                view! {
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
                                }
                                .into_any()
                            };

                            view! {
                                <tr class="align-top">
                                    {data_cells}
                                    {delete_cell}
                                </tr>
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
                    TableActionKind::ConfirmChanges => tr(lang.get(), "确认修改", "Confirm Changes"),
                    TableActionKind::AddEntries => tr(lang.get(), "添加多条", "Add Entries"),
                }}
            </button>
        </div>
    }
}

fn render_query_cell(
    idx: usize,
    entry: WordBankEntry,
    col_idx: usize,
    config: ColumnConfig,
    lang: ReadSignal<UiLanguage>,
    search_columns: ReadSignal<Vec<bool>>,
    on_begin_selected_drag: Callback<(usize, bool)>,
    on_drag_over_selected: Callback<(usize, leptos::ev::MouseEvent)>,
) -> AnyView {
    match config.editor_kind {
        ColumnEditorKind::SelectedToggle => view! {
            <td
                class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
                class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
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
                <input type="checkbox" prop:checked=entry.selected class="pointer-events-none"/>
            </td>
        }
        .into_any(),
        _ => view! {
            <td
                class="border border-slate-800 p-2"
                class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
            >
                {column_display_text(&entry, config.key, lang.get())}
            </td>
        }
        .into_any(),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_edit_cell(
    idx: usize,
    entry: WordBankEntry,
    col_idx: usize,
    config: ColumnConfig,
    lang: ReadSignal<UiLanguage>,
    search_columns: ReadSignal<Vec<bool>>,
    on_begin_selected_drag: Callback<(usize, bool)>,
    on_drag_over_selected: Callback<(usize, leptos::ev::MouseEvent)>,
    set_entries: WriteSignal<Vec<WordBankEntry>>,
    set_row_undo: WriteSignal<Vec<Option<WordBankEntry>>>,
    set_status: WriteSignal<String>,
    on_mark_hash_dirty: Callback<String>,
) -> AnyView {
    match config.editor_kind {
        ColumnEditorKind::ReadOnly => view! {
            <td
                class="border border-slate-800 p-1"
                class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
            >
                <span class="block rounded border border-slate-800 bg-slate-950 px-2 py-1 text-xs text-slate-300">
                    {entry.id.clone()}
                </span>
            </td>
        }
        .into_any(),
        ColumnEditorKind::SelectedToggle => view! {
            <td
                class="cursor-pointer select-none border border-slate-800 p-1 text-center hover:bg-slate-800/40"
                class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
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
                <input type="checkbox" prop:checked=entry.selected class="pointer-events-none"/>
            </td>
        }
        .into_any(),
        ColumnEditorKind::PartOfSpeechSelect => view! {
            <td
                class="border border-slate-800 p-1"
                class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
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
                                    tr(
                                        lang.get_untracked(),
                                        "词性更新失败：",
                                        "Failed to update part of speech:"
                                    )
                                ));
                                return;
                            }
                        };
                        set_entries.update(|list| {
                            if let Some(item) = list.get_mut(idx) {
                                snapshot_row_for_undo(idx, item, set_row_undo);
                                item.part_of_speech = parsed;
                                if config.hash_field {
                                    on_mark_hash_dirty.run(item.id.clone());
                                }
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
        }
        .into_any(),
        ColumnEditorKind::PipeListText
        | ColumnEditorKind::RequiredText
        | ColumnEditorKind::OptionalText => {
            let key = config.key;
            let initial_value = column_input_value(&entry, key);
            view! {
                <td
                    class="border border-slate-800 p-1"
                    class:hidden=move || !search_columns.get().get(col_idx).copied().unwrap_or(false)
                >
                    <input
                        type="text"
                        prop:value=initial_value
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_entries.update(|list| {
                                if let Some(item) = list.get_mut(idx) {
                                    snapshot_row_for_undo(idx, item, set_row_undo);
                                    apply_column_input(item, key, config.editor_kind, &value);
                                    if config.hash_field {
                                        on_mark_hash_dirty.run(item.id.clone());
                                    }
                                }
                            });
                        }
                        class="w-full rounded border border-slate-700 bg-slate-950 px-2 py-1"
                    />
                </td>
            }
            .into_any()
        }
    }
}

fn build_column_configs() -> Vec<ColumnConfig> {
    DATA_COLUMN_KEYS
        .iter()
        .map(|key| ColumnConfig {
            key,
            editor_kind: editor_kind_for_key(key),
            hash_field: field_meta(key)
                .map(|meta| meta.hash_relevant)
                .unwrap_or(false),
        })
        .collect()
}

fn editor_kind_for_key(key: &str) -> ColumnEditorKind {
    match key {
        "id" => ColumnEditorKind::ReadOnly,
        "selected" => ColumnEditorKind::SelectedToggle,
        "part_of_speech" => ColumnEditorKind::PartOfSpeechSelect,
        "tags" | "english" | "chinese" => ColumnEditorKind::PipeListText,
        "base_form" => ColumnEditorKind::RequiredText,
        _ => ColumnEditorKind::OptionalText,
    }
}

fn column_header_text(lang: UiLanguage, key: &str) -> String {
    if key == "delete" {
        tr(lang, "删除", "Delete").to_string()
    } else if key == "tags" || key == "english" || key == "chinese" {
        format!("{}(|)", field_label(lang, key))
    } else {
        field_label(lang, key).to_string()
    }
}

fn column_display_text(entry: &WordBankEntry, key: &str, lang: UiLanguage) -> String {
    if key == "part_of_speech" {
        entry.part_of_speech.display_name(lang).to_string()
    } else {
        entry_field_value(entry, key)
    }
}

fn column_input_value(entry: &WordBankEntry, key: &str) -> String {
    entry_field_value(entry, key)
}

fn apply_column_input(
    entry: &mut WordBankEntry,
    key: &str,
    editor_kind: ColumnEditorKind,
    raw: &str,
) {
    match editor_kind {
        ColumnEditorKind::PipeListText => match key {
            "tags" => entry.tags = parse_pipe_list(raw),
            "english" => entry.english = parse_pipe_list(raw),
            "chinese" => entry.chinese = parse_pipe_list(raw),
            _ => {}
        },
        ColumnEditorKind::RequiredText => {
            if key == "base_form" {
                entry.base_form = raw.to_string();
            }
        }
        ColumnEditorKind::OptionalText => {
            set_optional_field(entry, key, input_to_option(raw));
        }
        _ => {}
    }
}

fn set_optional_field(entry: &mut WordBankEntry, key: &str, value: Option<String>) {
    match key {
        "verb_present_tense" => entry.verb_present_tense = value,
        "verb_past_tense" => entry.verb_past_tense = value,
        "verb_imperative" => entry.verb_imperative = value,
        "noun_plural" => entry.noun_plural = value,
        "noun_singular_definite" => entry.noun_singular_definite = value,
        "noun_plural_definite" => entry.noun_plural_definite = value,
        "adjective_neuter_form" => entry.adjective_neuter_form = value,
        "adjective_plural_form" => entry.adjective_plural_form = value,
        "adjective_comparative" => entry.adjective_comparative = value,
        "adjective_superlative_indefinite" => entry.adjective_superlative_indefinite = value,
        "adjective_superlative_definite" => entry.adjective_superlative_definite = value,
        "adverb_comparative" => entry.adverb_comparative = value,
        "adverb_superlative" => entry.adverb_superlative = value,
        "verb_present_participle" => entry.verb_present_participle = value,
        "verb_past_participle" => entry.verb_past_participle = value,
        "verb_passive_infinitive" => entry.verb_passive_infinitive = value,
        "verb_passive_present" => entry.verb_passive_present = value,
        "verb_passive_past" => entry.verb_passive_past = value,
        "noun_singular_definite_genitive" => entry.noun_singular_definite_genitive = value,
        "noun_plural_definite_genitive" => entry.noun_plural_definite_genitive = value,
        "noun_singular_indefinite_genitive" => entry.noun_singular_indefinite_genitive = value,
        "noun_plural_indefinite_genitive" => entry.noun_plural_indefinite_genitive = value,
        "adjective_feminine_form" => entry.adjective_feminine_form = value,
        "pronoun_object" => entry.pronoun_object = value,
        "pronoun_reflexive" => entry.pronoun_reflexive = value,
        "pronoun_plural_subject" => entry.pronoun_plural_subject = value,
        "pronoun_plural_object" => entry.pronoun_plural_object = value,
        "pronoun_plural_reflexive" => entry.pronoun_plural_reflexive = value,
        "determinative_feminine_form" => entry.determinative_feminine_form = value,
        "determinative_neuter_form" => entry.determinative_neuter_form = value,
        "determinative_plural_form" => entry.determinative_plural_form = value,
        _ => {}
    }
}

fn snapshot_row_for_undo(
    idx: usize,
    item: &WordBankEntry,
    set_row_undo: WriteSignal<Vec<Option<WordBankEntry>>>,
) {
    let snapshot = item.clone();
    set_row_undo.update(|undo| {
        if undo.len() <= idx {
            undo.resize(idx + 1, None);
        }
        undo[idx] = Some(snapshot);
    });
}
