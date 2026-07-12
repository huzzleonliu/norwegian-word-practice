use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::utils::i18n::{field_label, tr};

pub const NONE_FIELD_KEY: &str = "none";

const PROMPT_FIELD_OPTIONS: [&str; 19] = [
    NONE_FIELD_KEY,
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

const ANSWER_FIELD_OPTIONS: [&str; 16] = [
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

pub fn default_answer_fields() -> Vec<String> {
    ANSWER_FIELD_OPTIONS
        .iter()
        .filter_map(|key| {
            if *key == "english" || *key == "chinese" {
                None
            } else {
                Some((*key).to_string())
            }
        })
        .collect()
}

#[component]
pub fn PracticeSettings(
    questions_per_page: ReadSignal<usize>,
    set_questions_per_page: WriteSignal<usize>,
    prompt_field_a: ReadSignal<String>,
    set_prompt_field_a: WriteSignal<String>,
    prompt_field_b: ReadSignal<String>,
    set_prompt_field_b: WriteSignal<String>,
    answer_fields: ReadSignal<Vec<String>>,
    set_answer_fields: WriteSignal<Vec<String>>,
    allow_answer_reveal: ReadSignal<bool>,
    set_allow_answer_reveal: WriteSignal<bool>,
    hide_all_revealed_answers: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    view! {
        <div class="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-3">
            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">
                    {move || tr(lang.get(), "一页练习（词条数）", "Entries per page")}
                </span>
                <input
                    type="number"
                    min="1"
                    prop:value=move || questions_per_page.get().to_string()
                    on:input=move |ev| {
                        let raw = event_target_value(&ev);
                        if let Ok(value) = raw.parse::<usize>() {
                            set_questions_per_page.set(value.max(1));
                        }
                    }
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
            </label>

            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">
                    {move || tr(lang.get(), "根据（下拉框 1）", "Prompt field (select 1)")}
                </span>
                <select
                    prop:value=move || prompt_field_a.get()
                    on:change=move |ev| set_prompt_field_a.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {PROMPT_FIELD_OPTIONS
                        .iter()
                        .map(|key| {
                            view! { <option value=*key>{move || field_label(lang.get(), key)}</option> }
                        })
                        .collect_view()}
                </select>
            </label>

            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">
                    {move || tr(lang.get(), "根据（下拉框 2）", "Prompt field (select 2)")}
                </span>
                <select
                    prop:value=move || prompt_field_b.get()
                    on:change=move |ev| set_prompt_field_b.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {PROMPT_FIELD_OPTIONS
                        .iter()
                        .map(|key| {
                            view! { <option value=*key>{move || field_label(lang.get(), key)}</option> }
                        })
                        .collect_view()}
                </select>
            </label>
        </div>

        <div class="mt-4">
            <p class="mb-2 text-sm text-slate-300">
                {move || tr(lang.get(), "回答（勾选要回答的项）", "Answers (select fields to answer)")}
            </p>
            <div class="space-y-3">
                {[
                    ("core", vec![0_usize, 1, 2]),
                    ("verb", vec![3, 4, 5]),
                    ("noun", vec![6, 7, 8]),
                    ("adjective", vec![9, 10, 11, 12, 13]),
                    ("adverb", vec![14, 15]),
                ]
                    .into_iter()
                    .map(|(group_name, indices)| {
                        let group_name_key = group_name;
                        view! {
                            <section class="rounded border border-slate-800 bg-slate-950/40 p-2">
                                <p class="mb-2 text-xs font-semibold text-slate-400">
                                    {move || match group_name_key {
                                        "core" => tr(lang.get(), "基础组", "Core"),
                                        "verb" => tr(lang.get(), "动词变体组", "Verb Forms"),
                                        "noun" => tr(lang.get(), "名词变体组", "Noun Forms"),
                                        "adjective" => tr(lang.get(), "形容词变体组", "Adjective Forms"),
                                        "adverb" => tr(lang.get(), "副词变体组", "Adverb Forms"),
                                        _ => group_name_key,
                                    }}
                                </p>
                                <div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-4">
                                    {indices
                                        .into_iter()
                                        .map(|idx| {
                                            let key = ANSWER_FIELD_OPTIONS[idx];
                                            let key_for_checked = key.to_string();
                                            let key_for_change = key.to_string();
                                            view! {
                                                <label class="inline-flex items-center gap-2 rounded border border-slate-800 bg-slate-900 px-2 py-1 text-xs text-slate-300">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || {
                                                            answer_fields
                                                                .get()
                                                                .iter()
                                                                .any(|field| field == &key_for_checked)
                                                        }
                                                        on:change=move |ev| {
                                                            let checked = event_target_checked(&ev);
                                                            let key_to_toggle = key_for_change.clone();
                                                            set_answer_fields.update(|fields| {
                                                                if checked {
                                                                    if !fields.iter().any(|field| field == &key_to_toggle) {
                                                                        fields.push(key_to_toggle.clone());
                                                                    }
                                                                } else {
                                                                    fields.retain(|field| field != &key_to_toggle);
                                                                }
                                                            });
                                                        }
                                                    />
                                                    <span>{move || field_label(lang.get(), key)}</span>
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
        </div>

        <div class="mt-4 rounded border border-slate-800 bg-slate-950/40 p-3">
            <p class="text-sm text-slate-300">{move || tr(lang.get(), "答案显隐", "Answer Reveal")}</p>
            <div class="mt-2 flex flex-wrap items-center gap-3">
                <label class="inline-flex items-center gap-2 text-xs text-slate-300">
                    <input
                        type="checkbox"
                        prop:checked=move || allow_answer_reveal.get()
                        on:change=move |ev| set_allow_answer_reveal.set(event_target_checked(&ev))
                    />
                    <span>
                        {move || {
                            tr(
                                lang.get(),
                                "允许在练习区临时亮出答案（默认隐藏）",
                                "Allow temporary answer reveal (hidden by default)",
                            )
                        }}
                    </span>
                </label>
                <button
                    type="button"
                    on:click=move |_| hide_all_revealed_answers.run(())
                    class="rounded border border-slate-700 bg-slate-900 px-2 py-1 text-xs text-slate-200 hover:bg-slate-800"
                >
                    {move || tr(lang.get(), "隐藏所有已亮出的答案", "Hide all revealed answers")}
                </button>
            </div>
        </div>
    }
}
