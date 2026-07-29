//! 列分组勾选面板：搜索范围与表格显示列共用同一布局。

use leptos::prelude::*;

use super::utils::{header_name, search_column_groups};
use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::{field_label, tr};

#[component]
pub fn ColumnGroupCheckboxes(
    lang: ReadSignal<UiLanguage>,
    columns: ReadSignal<Vec<bool>>,
    set_columns: WriteSignal<Vec<bool>>,
) -> impl IntoView {
    view! {
        <div class="space-y-3">
            {search_column_groups()
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
                                    "determinative" => {
                                        tr(lang.get(), "限定词变体组", "Determinative Forms")
                                    }
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
                                                        columns.get().get(idx).copied().unwrap_or(false)
                                                    }
                                                    on:change=move |ev| {
                                                        let checked = event_target_checked(&ev);
                                                        set_columns.update(|cols| {
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
}
