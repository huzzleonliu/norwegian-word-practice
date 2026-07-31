//! 练习题渲染模块：
//! - 题目列表渲染与输入控件
//! - 输入键生成与可答性判断
//! - 按 active id 稳定构建题目顺序
//! - 焦点激活词条并滚动到视口中心

use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::UiState;
use crate::structures::field_meta::{
    NONE_FIELD_KEY, entry_field_value as entry_field_value_from_meta,
};
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::{field_label, tr};

#[component]
pub fn CheckPracticeButton(
    on_check: Callback<()>,
    #[prop(optional)] label: Option<String>,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
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
    wrong_answer_feedback: Option<ReadSignal<HashMap<String, String>>>,
    set_wrong_answer_feedback: Option<WriteSignal<HashMap<String, String>>>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let (active_entry_id, set_active_entry_id) = signal(None::<String>);

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
                    let entry_id_for_focus = entry.id.clone();
                    let entry_id_for_active = entry.id.clone();
                    let entry_dom_id = practice_entry_dom_id(&entry.id);
                    view! {
                        <article
                            id=entry_dom_id.clone()
                            class=move || {
                                let is_active = active_entry_id
                                    .get()
                                    .as_ref()
                                    .is_some_and(|id| id == &entry_id_for_active);
                                if is_active {
                                    "rounded-lg border border-emerald-500 bg-slate-900/80 p-4 ring-1 ring-emerald-500/40 transition-colors"
                                        .to_string()
                                } else {
                                    "rounded-lg border border-slate-800 bg-slate-900/50 p-4 transition-colors"
                                        .to_string()
                                }
                            }
                            on:focusin=move |_| {
                                let id = entry_id_for_focus.clone();
                                let prev = active_entry_id.get_untracked();
                                if prev.as_ref() != Some(&id) {
                                    set_active_entry_id.set(Some(id.clone()));
                                    scroll_practice_entry_into_center(&id);
                                }
                            }
                        >
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
                                                let key_for_wrong = key_for_value.clone();
                                                let key_for_wrong_class = key_for_value.clone();
                                                let key_for_wrong_clear = key_for_value.clone();
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
                                                                            tabindex="-1"
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
                                                                if let Some(set_feedback) = set_wrong_answer_feedback {
                                                                    set_feedback.update(|feedback| {
                                                                        feedback.remove(&key_for_wrong_clear);
                                                                    });
                                                                }
                                                            }
                                                            placeholder=move || tr(lang.get(), "填写答案", "Type your answer")
                                                            class=move || {
                                                                let has_wrong = wrong_answer_feedback
                                                                    .map(|signal| signal.get().contains_key(&key_for_wrong_class))
                                                                    .unwrap_or(false);
                                                                if has_wrong {
                                                                    "rounded border border-red-500 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                                        .to_string()
                                                                } else {
                                                                    "rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                                        .to_string()
                                                                }
                                                            }
                                                        />
                                                        {move || {
                                                            let wrong_text = wrong_answer_feedback
                                                                .and_then(|signal| {
                                                                    signal.get().get(&key_for_wrong).cloned()
                                                                });
                                                            if let Some(wrong) = wrong_text {
                                                                let display = if wrong.trim().is_empty() {
                                                                    tr(lang.get(), "（空）", "(empty)").to_string()
                                                                } else {
                                                                    wrong
                                                                };
                                                                view! {
                                                                    <span class="text-[11px] text-red-400">
                                                                        {format!(
                                                                            "{}: {}",
                                                                            tr(lang.get(), "错误拼写", "Wrong spelling"),
                                                                            display
                                                                        )}
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }
                                                        }}
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

fn practice_entry_dom_id(entry_id: &str) -> String {
    format!("practice-entry-{entry_id}")
}

fn scroll_practice_entry_into_center(entry_id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(element) = document.get_element_by_id(&practice_entry_dom_id(entry_id)) else {
            return;
        };
        let options = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&options, &"block".into(), &"center".into());
        let _ = js_sys::Reflect::set(&options, &"behavior".into(), &"smooth".into());
        if let Ok(func) = js_sys::Reflect::get(&element, &"scrollIntoView".into()) {
            if let Ok(func) = func.dyn_into::<js_sys::Function>() {
                let _ = func.call1(&element, &options);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = entry_id;
    }
}

/// 根据 active id 顺序构建题目，确保 UI 展示顺序稳定。
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

/// 生成输入框键：`{entry_id}::{field}`。
pub fn answer_input_key(entry_id: &str, field: &str) -> String {
    format!("{entry_id}::{field}")
}

/// 判断字段在当前词条上是否可作答（目标值非空）。
pub fn is_answer_field_available(entry: &WordBankEntry, field: &str) -> bool {
    !entry_field_value(entry, field).trim().is_empty()
}

/// 按字段 key 读取词条值，统一返回字符串用于显示与判题。
pub fn entry_field_value(entry: &WordBankEntry, field: &str) -> String {
    entry_field_value_from_meta(entry, field)
}
