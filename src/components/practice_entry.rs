use std::collections::HashMap;

use leptos::prelude::*;

use crate::components::lexicon_browser::WordEntry;
use crate::components::practice_settings::NONE_FIELD_KEY;

#[component]
pub fn PracticeEntry(
    active_question_ids: ReadSignal<Vec<String>>,
    entries: ReadSignal<Vec<WordEntry>>,
    prompt_field_a: ReadSignal<String>,
    prompt_field_b: ReadSignal<String>,
    answer_fields: ReadSignal<Vec<String>>,
    answer_inputs: ReadSignal<HashMap<String, String>>,
    set_answer_inputs: WriteSignal<HashMap<String, String>>,
) -> impl IntoView {
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
                                {format!("第 {} 题", index + 1)}
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
                                                "{}：{}",
                                                field,
                                                entry_field_value(&entry_for_prompt, &field)
                                            )
                                        })
                                        .collect::<Vec<_>>();
                                    if prompts.is_empty() {
                                        "根据：无".to_string()
                                    } else {
                                        format!("根据：{}", prompts.join(" | "))
                                    }
                                }}
                            </div>

                            <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
                                {move || {
                                    let selected_answer_fields = answer_fields.get();
                                    if selected_answer_fields.is_empty() {
                                        view! {
                                            <p class="text-xs text-amber-300">
                                                "请在上方至少勾选 1 个“回答”项。"
                                            </p>
                                        }
                                            .into_any()
                                    } else {
                                        selected_answer_fields
                                            .into_iter()
                                            .map(|field| {
                                                let entry_id = entry_for_answers.id.clone();
                                                let field_for_label = field.clone();
                                                let field_for_key = field.clone();
                                                let expected = entry_field_value(&entry_for_answers, &field);
                                                let key_for_value = answer_input_key(&entry_id, &field_for_key);
                                                let key_for_input = key_for_value.clone();
                                                view! {
                                                    <label class="flex flex-col gap-1 text-xs text-slate-300">
                                                        <span>{field_for_label}</span>
                                                        <input
                                                            type="text"
                                                            prop:value=move || {
                                                                answer_inputs
                                                                    .get()
                                                                    .get(&key_for_value)
                                                                    .cloned()
                                                                    .unwrap_or_default()
                                                            }
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                set_answer_inputs.update(|inputs| {
                                                                    inputs.insert(key_for_input.clone(), value);
                                                                });
                                                            }
                                                            placeholder=format!("参考值：{expected}")
                                                            class="rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                        />
                                                    </label>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
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
                        "当前没有可练习题目，请返回上页先选择词条。"
                    </p>
                }
                    .into_any()
            } else {
                view! { <></> }.into_any()
            }
        }}
    }
}

pub fn build_question_items(
    active_ids: &[String],
    entries: &[WordEntry],
) -> Vec<(usize, WordEntry)> {
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

pub fn entry_field_value(entry: &WordEntry, field: &str) -> String {
    match field {
        "id" => entry.id.clone(),
        "part_of_speech" => entry.part_of_speech.clone(),
        "tags" => entry.tags.join(" | "),
        "english" => entry.english.join(" | "),
        "chinese" => entry.chinese.join(" | "),
        "base_form" => entry.base_form.clone(),
        "past_tense" => entry.past_tense.clone().unwrap_or_default(),
        "imperative" => entry.imperative.clone().unwrap_or_default(),
        "plural" => entry.plural.clone().unwrap_or_default(),
        "singular_definite" => entry.singular_definite.clone().unwrap_or_default(),
        "plural_definite" => entry.plural_definite.clone().unwrap_or_default(),
        "neuter_form" => entry.neuter_form.clone().unwrap_or_default(),
        "plural_form" => entry.plural_form.clone().unwrap_or_default(),
        "adjective_comparative" => entry.adjective_comparative.clone().unwrap_or_default(),
        "adjective_superlative_indefinite" => entry
            .adjective_superlative_indefinite
            .clone()
            .unwrap_or_default(),
        "adjective_superlative_definite" => entry
            .adjective_superlative_definite
            .clone()
            .unwrap_or_default(),
        "adverb_comparative" => entry.adverb_comparative.clone().unwrap_or_default(),
        "adverb_superlative" => entry.adverb_superlative.clone().unwrap_or_default(),
        _ => String::new(),
    }
}
