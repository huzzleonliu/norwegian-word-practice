use leptos::prelude::*;

use super::utils::{header_name, DATA_COLUMN_KEYS};
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::{field_label, tr};

#[component]
pub fn LexiconSearchPanel(
    lang: ReadSignal<UiLanguage>,
    search_text: ReadSignal<String>,
    set_search_text: WriteSignal<String>,
    search_columns: ReadSignal<Vec<bool>>,
    set_search_columns: WriteSignal<Vec<bool>>,
    search_scope_expanded: ReadSignal<bool>,
    set_search_scope_expanded: WriteSignal<bool>,
    on_apply_search: Callback<()>,
) -> impl IntoView {
    const DATA_COLUMN_COUNT: usize = DATA_COLUMN_KEYS.len();

    view! {
        <div class="mb-4 rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                <div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
                    <input
                        type="text"
                        placeholder=move || tr(lang.get(), "输入要查找的字符", "Type to search")
                        prop:value=move || search_text.get()
                        on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        class="w-full min-w-0 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm sm:min-w-60 sm:flex-1"
                    />
                    <button
                        type="button"
                        on:click=move |_| on_apply_search.run(())
                        class="w-full rounded border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "查找", "Search")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![true; DATA_COLUMN_COUNT])
                        class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700 sm:w-auto"
                    >
                        {move || tr(lang.get(), "全选列", "Select all columns")}
                    </button>
                    <button
                        type="button"
                        on:click=move |_| set_search_columns.set(vec![false; DATA_COLUMN_COUNT])
                        class="w-full rounded border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium hover:bg-slate-700 sm:w-auto"
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
                        class="inline-flex w-full items-center justify-center gap-2 rounded border border-slate-700 bg-slate-900 px-3 py-1 text-xs font-medium text-slate-200 hover:bg-slate-800 sm:w-auto sm:justify-start"
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
                                        ("verb", vec![7, 8, 9, 20, 21, 22, 23, 24]),
                                        ("noun", vec![10, 11, 12, 25, 26, 27, 28]),
                                        ("adjective", vec![29, 13, 14, 15, 16, 17]),
                                        ("pronoun", vec![30, 31, 32, 33, 34]),
                                        ("determinative", vec![35, 36, 37]),
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
                                                            "pronoun" => tr(lang.get(), "代词变体组", "Pronoun Forms"),
                                                            "determinative" => tr(lang.get(), "限定词变体组", "Determinative Forms"),
                                                            "adverb" => tr(lang.get(), "副词变体组", "Adverb Forms"),
                                                            _ => group_name,
                                                        }}
                                                    </p>
                                                    <div class="grid grid-cols-1 gap-2 sm:grid-cols-2 md:grid-cols-4 lg:grid-cols-5">
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
    }
}
