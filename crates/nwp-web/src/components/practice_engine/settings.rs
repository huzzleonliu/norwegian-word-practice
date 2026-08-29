//! 练习设置模块：
//! - 配置每页题量
//! - 配置提示字段与回答字段
//! - 控制答案显隐策略

use leptos::prelude::*;

use crate::app_state::UiState;
use crate::structures::field_meta::{
    ANSWER_FIELD_GROUPS, ANSWER_FIELD_OPTIONS, PROMPT_FIELD_OPTIONS,
};
use crate::utils::i18n::{field_label, tr};

/// 默认回答字段：与「常用」预设一致。
pub fn default_answer_fields() -> Vec<String> {
    AnswerPreset::Common.fields()
}

/// 回答字段预设：基础 / 常用 / 全面。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnswerPreset {
    Basic,
    Common,
    Full,
}

impl AnswerPreset {
    fn as_key(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Common => "common",
            Self::Full => "full",
        }
    }

    fn from_key(raw: &str) -> Option<Self> {
        match raw {
            "basic" => Some(Self::Basic),
            "common" => Some(Self::Common),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    fn fields(self) -> Vec<String> {
        match self {
            Self::Basic => vec!["base_form".to_string()],
            Self::Common => ANSWER_FIELDS_COMMON
                .iter()
                .map(|key| (*key).to_string())
                .collect(),
            Self::Full => ANSWER_FIELD_OPTIONS
                .iter()
                .map(|key| (*key).to_string())
                .collect(),
        }
    }
}

/// 常用预设：原型及核心变体。
const ANSWER_FIELDS_COMMON: &[&str] = &[
    "base_form",
    "verb_present_tense",
    "verb_past_tense",
    "verb_past_participle",
    "noun_singular_definite",
    "noun_plural",
    "adjective_neuter_form",
    "adjective_plural_form",
    "adjective_comparative",
    "adjective_superlative_indefinite",
    "determinative_neuter_form",
    "determinative_plural_form",
    "adverb_comparative",
    "adverb_superlative",
];

#[component]
pub fn PracticeSettings(
    questions_per_page: ReadSignal<usize>,
    set_questions_per_page: WriteSignal<usize>,
    prompt_field_a: ReadSignal<String>,
    set_prompt_field_a: WriteSignal<String>,
    prompt_field_b: ReadSignal<String>,
    set_prompt_field_b: WriteSignal<String>,
    prompt_field_c: ReadSignal<String>,
    set_prompt_field_c: WriteSignal<String>,
    answer_fields: ReadSignal<Vec<String>>,
    set_answer_fields: WriteSignal<Vec<String>>,
    allow_answer_reveal: ReadSignal<bool>,
    set_allow_answer_reveal: WriteSignal<bool>,
    hide_all_revealed_answers: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let (answers_expanded, set_answers_expanded) = signal(false);
    let (answer_preset, set_answer_preset) = signal(AnswerPreset::Common.as_key().to_string());

    view! {
        <div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
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

            <label class="flex flex-col gap-1 text-sm">
                <span class="text-slate-300">
                    {move || tr(lang.get(), "根据（下拉框 3）", "Prompt field (select 3)")}
                </span>
                <select
                    prop:value=move || prompt_field_c.get()
                    on:change=move |ev| set_prompt_field_c.set(event_target_value(&ev))
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
            <div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
                <p class="text-sm text-slate-300">
                    {move || {
                        tr(
                            lang.get(),
                            "回答（勾选要回答的项）",
                            "Answers (select fields to answer)",
                        )
                    }}
                </p>
                <label class="inline-flex items-center gap-2 text-xs text-slate-300">
                    <span class="shrink-0">{move || tr(lang.get(), "预设", "Preset")}</span>
                    <select
                        prop:value=move || answer_preset.get()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_answer_preset.set(value.clone());
                            if let Some(preset) = AnswerPreset::from_key(&value) {
                                set_answer_fields.set(preset.fields());
                                set_answers_expanded.set(false);
                            }
                        }
                        class="min-w-28 rounded border border-slate-700 bg-slate-900 px-2 py-1.5 text-xs"
                    >
                        <option value=AnswerPreset::Basic.as_key()>
                            {move || tr(lang.get(), "基础", "Basic")}
                        </option>
                        <option value=AnswerPreset::Common.as_key()>
                            {move || tr(lang.get(), "常用", "Common")}
                        </option>
                        <option value=AnswerPreset::Full.as_key()>
                            {move || tr(lang.get(), "全面", "Full")}
                        </option>
                    </select>
                </label>
                <button
                    type="button"
                    on:click=move |_| {
                        set_answers_expanded.update(|expanded| *expanded = !*expanded)
                    }
                    class="inline-flex w-full items-center justify-center gap-2 rounded border border-slate-700 bg-slate-900 px-3 py-1.5 text-xs font-medium text-slate-200 hover:bg-slate-800 sm:w-auto sm:justify-start"
                >
                    {move || {
                        if answers_expanded.get() {
                            tr(lang.get(), "隐藏自定义回答项", "Hide Custom Answers")
                        } else {
                            tr(lang.get(), "自定义回答项", "Custom Answer Fields")
                        }
                    }}
                </button>
            </div>

            {move || {
                if answers_expanded.get() {
                    view! {
                        <div class="mt-3 space-y-3">
                            {ANSWER_FIELD_GROUPS
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
                                                    "adjective" => {
                                                        tr(lang.get(), "形容词变体组", "Adjective Forms")
                                                    }
                                                    "pronoun" => {
                                                        tr(lang.get(), "代词变体组", "Pronoun Forms")
                                                    }
                                                    "determinative" => {
                                                        tr(lang.get(), "限定词变体组", "Determinative Forms")
                                                    }
                                                    "adverb" => {
                                                        tr(lang.get(), "副词变体组", "Adverb Forms")
                                                    }
                                                    _ => group_name_key,
                                                }}
                                            </p>
                                            <div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-4">
                                                {indices
                                                    .into_iter()
                                                    .map(|key| {
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
                                                                                if !fields
                                                                                    .iter()
                                                                                    .any(|field| field == &key_to_toggle)
                                                                                {
                                                                                    fields.push(key_to_toggle.clone());
                                                                                }
                                                                            } else {
                                                                                fields.retain(|field| {
                                                                                    field != &key_to_toggle
                                                                                });
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
                    }
                        .into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
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
