use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::practice_settings::NONE_FIELD_KEY;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::{field_label, tr};

#[component]
pub fn CheckPracticeButton(
    on_check: Callback<()>,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    let check_click = move |_| on_check.run(());
    let label = label.unwrap_or_else(|| tr(lang.get_untracked(), "检查", "Check").to_string());
    let class = class.unwrap_or_else(|| {
        "w-full rounded-lg border border-emerald-600 bg-emerald-700 px-4 py-3 text-sm font-semibold text-white hover:bg-emerald-600 sm:w-auto"
            .to_string()
    });

    view! {
        <button type="button" on:click=check_click class=class>
            {label}
        </button>
    }
}

#[component]
pub fn PracticeEntry(
    on_check: Callback<()>,
    active_question_ids: ReadSignal<Vec<String>>,
    entries: ReadSignal<Vec<WordBankEntry>>,
    prompt_field_a: ReadSignal<String>,
    prompt_field_b: ReadSignal<String>,
    answer_fields: ReadSignal<Vec<String>>,
    answer_inputs: ReadSignal<HashMap<String, String>>,
    set_answer_inputs: WriteSignal<HashMap<String, String>>,
    allow_answer_reveal: Option<ReadSignal<bool>>,
    revealed_answer_keys: Option<ReadSignal<HashSet<String>>>,
    set_revealed_answer_keys: Option<WriteSignal<HashSet<String>>>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    view! {
        <div class="mt-4 space-y-4">
            <For
                each=move || {
                    let active_ids = active_question_ids.get();
                    let all_entries = entries.get();
                    build_question_items(&active_ids, &all_entries)
                }
                key=|(_, entry)| entry.id.clone()
                children=move |(index, entry)| {
                    let entry_for_prompt = entry.clone();
                    let entry_for_answers = entry.clone();
                    view! {
                        <article class="rounded-lg border border-slate-800 bg-slate-900/50 p-4">
                            <p class="text-sm font-semibold text-slate-200">
                                {move || {
                                    format!(
                                        "{} {}",
                                        tr(lang.get(), "第", "Question"),
                                        index + 1
                                    )
                                }}
                            </p>

                            <div class="mt-2 space-y-1 text-sm text-slate-300">
                                {move || {
                                    let field_a = prompt_field_a.get();
                                    let field_b = prompt_field_b.get();
                                    let prompts = [field_a, field_b]
                                        .into_iter()
                                        .filter(|field| field.as_str() != NONE_FIELD_KEY)
                                        .map(|field| {
                                            format!(
                                                "{}: {}",
                                                field_label(lang.get(), &field),
                                                if field == "part_of_speech" {
                                                    entry_for_prompt
                                                        .part_of_speech
                                                        .display_name(lang.get())
                                                        .to_string()
                                                } else {
                                                    entry_field_value(&entry_for_prompt, &field)
                                                }
                                            )
                                        })
                                        .collect::<Vec<_>>();
                                    if prompts.is_empty() {
                                        format!(
                                            "{}: {}",
                                            tr(lang.get(), "根据", "Prompt"),
                                            tr(lang.get(), "无", "None")
                                        )
                                    } else {
                                        format!("{}: {}", tr(lang.get(), "根据", "Prompt"), prompts.join(" | "))
                                    }
                                }}
                            </div>

                            <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
                                {move || {
                                    let selected_answer_fields = answer_fields.get();
                                    if selected_answer_fields.is_empty() {
                                        view! {
                                            <p class="text-xs text-amber-300">
                                                {move || {
                                                    tr(
                                                        lang.get(),
                                                        "请在上方至少勾选 1 个“回答”项。",
                                                        "Please select at least one answer field above.",
                                                    )
                                                }}
                                            </p>
                                        }
                                            .into_any()
                                    } else {
                                        let available_answer_fields = selected_answer_fields
                                            .into_iter()
                                            .filter(|field| is_answer_field_available(&entry_for_answers, field))
                                            .collect::<Vec<_>>();
                                        if available_answer_fields.is_empty() {
                                            view! {
                                                <p class="text-xs text-slate-400">
                                                    {move || {
                                                        tr(
                                                            lang.get(),
                                                            "该词条在当前回答项下没有可作答字段。",
                                                            "This entry has no answerable fields under current settings.",
                                                        )
                                                    }}
                                                </p>
                                            }
                                                .into_any()
                                        } else {
                                            available_answer_fields
                                                .into_iter()
                                                .map(|field| {
                                                let entry_id = entry_for_answers.id.clone();
                                                let field_for_label = field.clone();
                                                let field_for_key = field.clone();
                                                let expected = entry_field_value(&entry_for_answers, &field);
                                                let key_for_value = answer_input_key(&entry_id, &field_for_key);
                                                let key_for_input = key_for_value.clone();
                                                let key_for_reveal_check = key_for_value.clone();
                                                let key_for_reveal_toggle = key_for_value.clone();
                                                let key_for_value_read = key_for_value.clone();
                                                let key_for_value_revealed = key_for_value.clone();
                                                view! {
                                                    <label class="flex flex-col gap-1 text-xs text-slate-300">
                                                        <div class="flex items-center justify-between gap-2">
                                                            <span>{move || field_label(lang.get(), &field_for_label)}</span>
                                                            {move || {
                                                                let allow_reveal = allow_answer_reveal
                                                                    .map(|signal| signal.get())
                                                                    .unwrap_or(false);
                                                                if allow_reveal {
                                                                    let is_revealed = revealed_answer_keys
                                                                        .map(|signal| signal.get().contains(&key_for_reveal_check))
                                                                        .unwrap_or(false);
                                                                    let label = if is_revealed {
                                                                        tr(lang.get(), "隐藏", "Hide")
                                                                    } else {
                                                                        tr(lang.get(), "显示", "Show")
                                                                    };
                                                                    let toggle_key = key_for_reveal_toggle.clone();
                                                                    view! {
                                                                        <button
                                                                            type="button"
                                                                            on:click=move |_| {
                                                                                let reveal_key = toggle_key.clone();
                                                                                if let Some(set_signal) = set_revealed_answer_keys {
                                                                                    set_signal.update(|keys| {
                                                                                        if keys.contains(&reveal_key) {
                                                                                            keys.remove(&reveal_key);
                                                                                        } else {
                                                                                            keys.insert(reveal_key);
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                            class="rounded border border-slate-700 bg-slate-900 px-2 py-0.5 text-[11px] text-slate-200 hover:bg-slate-800"
                                                                        >
                                                                            {label}
                                                                        </button>
                                                                    }
                                                                        .into_any()
                                                                } else {
                                                                    view! { <></> }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                        <input
                                                            type="text"
                                                            prop:value=move || {
                                                                answer_inputs
                                                                    .get()
                                                                    .get(&key_for_value_read)
                                                                    .cloned()
                                                                    .unwrap_or_default()
                                                            }
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                set_answer_inputs.update(|inputs| {
                                                                    inputs.insert(key_for_input.clone(), value);
                                                                });
                                                            }
                                                            placeholder=move || tr(lang.get(), "填写答案", "Type your answer")
                                                            class="rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                        />
                                                        {move || {
                                                            let allow_reveal = allow_answer_reveal
                                                                .map(|signal| signal.get())
                                                                .unwrap_or(false);
                                                            let is_revealed = revealed_answer_keys
                                                                .map(|signal| signal.get().contains(&key_for_value_revealed))
                                                                .unwrap_or(false);
                                                            if allow_reveal && is_revealed
                                                            {
                                                                view! {
                                                                    <span class="text-[11px] text-amber-300">
                                                                        {format!(
                                                                            "{}: {}",
                                                                            tr(lang.get(), "参考答案", "Reference"),
                                                                            expected.as_str()
                                                                        )}
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }
                                                        }}
                                                    </label>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                        }
                                    }
                                }}
                            </div>
                        </article>
                    }
                }
            />
        </div>

        {move || {
            if active_question_ids.get().is_empty() {
                view! {
                    <p class="mt-4 text-sm text-amber-300">
                        {move || {
                            tr(
                                lang.get(),
                                "当前没有可练习题目，请返回上页先选择词条。",
                                "No questions available. Please go back and select entries first.",
                            )
                        }}
                    </p>
                }
                    .into_any()
            } else {
                view! { <></> }.into_any()
            }
        }}

        <div class="mt-4 flex justify-center">
            <CheckPracticeButton
                on_check=on_check
                class="w-full rounded-lg border border-emerald-600 bg-emerald-700 px-4 py-3 text-sm font-semibold text-white hover:bg-emerald-600 sm:w-auto".to_string()
            />
        </div>
    }
}

pub fn build_question_items(
    active_ids: &[String],
    entries: &[WordBankEntry],
) -> Vec<(usize, WordBankEntry)> {
    active_ids
        .iter()
        .enumerate()
        .filter_map(|(index, id)| {
            entries
                .iter()
                .find(|entry| entry.id == *id)
                .cloned()
                .map(|entry| (index, entry))
        })
        .collect()
}

pub fn answer_input_key(entry_id: &str, field: &str) -> String {
    format!("{entry_id}::{field}")
}

pub fn is_answer_field_available(entry: &WordBankEntry, field: &str) -> bool {
    !entry_field_value(entry, field).trim().is_empty()
}

pub fn entry_field_value(entry: &WordBankEntry, field: &str) -> String {
    match field {
        "id" => entry.id.clone(),
        "part_of_speech" => entry.part_of_speech.as_key().to_string(),
        "tags" => entry.tags.join(" | "),
        "english" => entry.english.join(" | "),
        "chinese" => entry.chinese.join(" | "),
        "base_form" => entry.base_form.clone(),
        "verb_present_tense" => entry.verb_present_tense.clone().unwrap_or_default(),
        "verb_past_tense" => entry.verb_past_tense.clone().unwrap_or_default(),
        "verb_imperative" => entry.verb_imperative.clone().unwrap_or_default(),
        "verb_present_participle" => entry.verb_present_participle.clone().unwrap_or_default(),
        "verb_past_participle" => entry.verb_past_participle.clone().unwrap_or_default(),
        "verb_passive_infinitive" => entry.verb_passive_infinitive.clone().unwrap_or_default(),
        "verb_passive_present" => entry.verb_passive_present.clone().unwrap_or_default(),
        "verb_passive_past" => entry.verb_passive_past.clone().unwrap_or_default(),
        "noun_plural" => entry.noun_plural.clone().unwrap_or_default(),
        "noun_singular_definite" => entry.noun_singular_definite.clone().unwrap_or_default(),
        "noun_plural_definite" => entry.noun_plural_definite.clone().unwrap_or_default(),
        "noun_singular_definite_genitive" => entry
            .noun_singular_definite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_plural_definite_genitive" => entry
            .noun_plural_definite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_singular_indefinite_genitive" => entry
            .noun_singular_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        "noun_plural_indefinite_genitive" => entry
            .noun_plural_indefinite_genitive
            .clone()
            .unwrap_or_default(),
        "adjective_feminine_form" => entry.adjective_feminine_form.clone().unwrap_or_default(),
        "adjective_neuter_form" => entry.adjective_neuter_form.clone().unwrap_or_default(),
        "adjective_plural_form" => entry.adjective_plural_form.clone().unwrap_or_default(),
        "adjective_comparative" => entry.adjective_comparative.clone().unwrap_or_default(),
        "adjective_superlative_indefinite" => entry
            .adjective_superlative_indefinite
            .clone()
            .unwrap_or_default(),
        "adjective_superlative_definite" => entry
            .adjective_superlative_definite
            .clone()
            .unwrap_or_default(),
        "pronoun_object" => entry.pronoun_object.clone().unwrap_or_default(),
        "pronoun_reflexive" => entry.pronoun_reflexive.clone().unwrap_or_default(),
        "pronoun_plural_subject" => entry.pronoun_plural_subject.clone().unwrap_or_default(),
        "pronoun_plural_object" => entry.pronoun_plural_object.clone().unwrap_or_default(),
        "pronoun_plural_reflexive" => entry.pronoun_plural_reflexive.clone().unwrap_or_default(),
        "determinative_feminine_form" => entry
            .determinative_feminine_form
            .clone()
            .unwrap_or_default(),
        "determinative_neuter_form" => entry.determinative_neuter_form.clone().unwrap_or_default(),
        "determinative_plural_form" => entry.determinative_plural_form.clone().unwrap_or_default(),
        "adverb_comparative" => entry.adverb_comparative.clone().unwrap_or_default(),
        "adverb_superlative" => entry.adverb_superlative.clone().unwrap_or_default(),
        _ => String::new(),
    }
}
