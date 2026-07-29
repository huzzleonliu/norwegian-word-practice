//! 多条新增组件：解析批量输入并复用词库表格完成批量编辑后提交。

use leptos::prelude::*;

use crate::app_state::UiState;
use crate::components::lexicon_browser::parse_word_bank_csv;
use crate::components::lexicon_browser::table_panel::{LexiconTablePanel, TableActionKind};
use crate::components::lexicon_browser::utils::{
    DATA_COLUMN_KEYS, default_search_column_visibility,
};
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::dictionary::current_added_at_timestamp;
use crate::utils::i18n::tr;

#[component]
pub fn LexiconEditorAddMulti(
    on_submit: Callback<Vec<WordBankEntry>>,
    on_clear: Callback<()>,
    bulk_input: ReadSignal<String>,
    bulk_errors: ReadSignal<Vec<String>>,
    bulk_success_message: ReadSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();
    let (preview_entries, set_preview_entries) = signal(Vec::<WordBankEntry>::new());
    let (preview_baseline_entries, set_preview_baseline_entries) =
        signal(Vec::<WordBankEntry>::new());
    let (preview_row_undo, set_preview_row_undo) = signal(Vec::<Option<WordBankEntry>>::new());
    let (preview_delete_marks, set_preview_delete_marks) = signal(Vec::<bool>::new());
    let (preview_delete_drag_target, set_preview_delete_drag_target) = signal(None::<bool>);
    let (preview_col_widths, _set_preview_col_widths) = signal({
        let mut widths = vec![160_u16; DATA_COLUMN_COUNT + 1];
        if let Some(selected_col) = widths.get_mut(1) {
            *selected_col = 90;
        }
        if let Some(operation_col) = widths.get_mut(DATA_COLUMN_COUNT) {
            *operation_col = 220;
        }
        widths
    });
    let (preview_sort_state, _set_preview_sort_state) = signal(Vec::<(usize, bool)>::new());
    let (preview_search_columns, _set_preview_search_columns) =
        signal(default_search_column_visibility());
    let (preview_detected_count, set_preview_detected_count) = signal(0_usize);
    let (show_preview_table, set_show_preview_table) = signal(false);
    let (_preview_status, set_preview_status) = signal(String::new());
    let (table_confirm_error, set_table_confirm_error) = signal(String::new());
    let (table_confirm_success, set_table_confirm_success) = signal(String::new());

    // 输入源变化时重新解析预览数据；表格编辑本身不会回写 bulk_input。
    Effect::new(move |_| {
        let content = bulk_input.get();
        if content.trim().is_empty() {
            set_preview_detected_count.set(0);
            set_show_preview_table.set(false);
            set_preview_entries.set(Vec::new());
            set_preview_baseline_entries.set(Vec::new());
            set_preview_row_undo.set(Vec::new());
            set_preview_delete_marks.set(Vec::new());
            return;
        }
        match parse_word_bank_csv(&content) {
            Ok(rows) => {
                set_preview_detected_count.set(rows.len());
                set_show_preview_table.set(rows.len() > 1);
                set_preview_baseline_entries.set(rows.clone());
                set_preview_row_undo.set(vec![None; rows.len()]);
                set_preview_delete_marks.set(vec![false; rows.len()]);
                set_preview_entries.set(rows);
            }
            Err(_) => {
                set_preview_detected_count.set(0);
                set_show_preview_table.set(false);
                set_preview_entries.set(Vec::new());
                set_preview_baseline_entries.set(Vec::new());
                set_preview_row_undo.set(Vec::new());
                set_preview_delete_marks.set(Vec::new());
            }
        }
    });

    Effect::new(move |_| {
        set_table_confirm_error.set(bulk_errors.get().join("；"));
    });
    Effect::new(move |_| {
        set_table_confirm_success.set(bulk_success_message.get());
    });

    let preview_row_items = Signal::derive(move || {
        preview_entries
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, WordBankEntry)>>()
    });

    let noop_sort = Callback::new(|_: (usize, bool)| {});
    let noop_resize = Callback::new(|_: (usize, leptos::ev::MouseEvent)| {});
    let noop_selected_drag = Callback::new(|_: (usize, bool)| {});
    let noop_drag_over = Callback::new(|_: (usize, leptos::ev::MouseEvent)| {});
    let noop_click = Callback::new(|_: ()| {});
    // 预览表格删除列采用与主浏览器一致的拖拽批量勾选逻辑。
    let on_begin_delete_drag = Callback::new(move |(idx, current_checked): (usize, bool)| {
        let target_checked = !current_checked;
        set_preview_delete_drag_target.set(Some(target_checked));
        update_delete_mark(idx, target_checked, set_preview_delete_marks);
    });
    let on_drag_over_delete = Callback::new(move |(idx, ev): (usize, leptos::ev::MouseEvent)| {
        if ev.buttons() & 1 != 1 {
            return;
        }
        if let Some(target_checked) = preview_delete_drag_target.get_untracked() {
            update_delete_mark(idx, target_checked, set_preview_delete_marks);
        }
    });

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">
                {move || tr(lang.get(), "多条添加（CSV，多行）", "Bulk Add (CSV, multiple rows)")}
            </h2>

            {move || {
                if show_preview_table.get() {
                    view! {
                        <section class="mt-3 rounded-xl border border-slate-800 bg-slate-900/40 p-3 sm:p-4">
                            <p class="mb-2 text-xs text-slate-400">
                                {move || {
                                    format!(
                                        "{} {} {}",
                                        tr(lang.get(), "已检测到", "Detected"),
                                        preview_detected_count.get(),
                                        tr(lang.get(), "条词条，以下为表格预览。", "entries, showing table preview."),
                                    )
                                }}
                            </p>
                            <LexiconTablePanel
                                lang=lang
                                is_query_mode=false
                                search_columns=preview_search_columns
                                col_widths=preview_col_widths
                                sort_state=preview_sort_state
                                row_items=preview_row_items
                                entries=preview_entries
                                on_sort_by_column=noop_sort
                                on_start_resize=noop_resize
                                on_begin_selected_drag=noop_selected_drag
                                on_drag_over_selected=noop_drag_over
                                set_entries=set_preview_entries
                                set_row_undo=set_preview_row_undo
                                baseline_entries=preview_baseline_entries
                                set_baseline_entries=set_preview_baseline_entries
                                row_undo=preview_row_undo
                                set_status=set_preview_status
                                confirm_error=table_confirm_error
                                confirm_success=table_confirm_success
                                delete_marks=preview_delete_marks
                                on_begin_delete_drag=on_begin_delete_drag
                                on_drag_over_delete=on_drag_over_delete
                                on_select_visible_click=noop_click
                                on_unselect_visible_click=noop_click
                                on_confirm_changes=Callback::new(move |_| {
                                    let marks = preview_delete_marks.get_untracked();
                                    let now = current_added_at_timestamp();
                                    let rows = preview_entries
                                        .get_untracked()
                                        .into_iter()
                                        .enumerate()
                                        .filter_map(|(idx, mut entry)| {
                                            if marks.get(idx).copied().unwrap_or(false) {
                                                None
                                            } else {
                                                // 批量新增：空添加时间统一写入本批次时间戳
                                                if entry.added_at.trim().is_empty() {
                                                    entry.added_at = now.clone();
                                                }
                                                Some(entry)
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    on_submit.run(rows);
                                })
                                action_kind=TableActionKind::AddEntries
                                show_select_buttons=false
                                on_clear=on_clear
                            />
                        </section>
                    }
                        .into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
            {move || {
                let errors = bulk_errors.get();
                if errors.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <ul class="mt-3 list-disc space-y-1 pl-5 text-sm text-red-400">
                            {errors
                                .into_iter()
                                .map(|item| view! { <li>{item}</li> })
                                .collect_view()}
                        </ul>
                    }
                        .into_any()
                }
            }}
            {move || {
                let message = bulk_success_message.get();
                if message.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! { <p class="mt-3 text-sm font-medium text-emerald-400">{message}</p> }
                        .into_any()
                }
            }}
            {move || {
                if show_preview_table.get() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <p class="mt-3 text-sm text-slate-400">
                            {move || {
                                tr(
                                    lang.get(),
                                    "暂无可编辑的多条结果。请先使用上方“查询并分流”生成多条候选。",
                                    "No editable multi-results yet. Use \"Query and Route\" above first.",
                                )
                            }}
                        </p>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

fn update_delete_mark(idx: usize, checked: bool, set_delete_marks: WriteSignal<Vec<bool>>) {
    set_delete_marks.update(|marks| {
        if marks.len() <= idx {
            marks.resize(idx + 1, false);
        }
        marks[idx] = checked;
    });
}
