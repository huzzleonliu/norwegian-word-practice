use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::WordEntry;
use crate::pages::AppPage;
use crate::structures::pracresult::{
    AnswerStats, MAX_WRONG_ANSWERS_PER_FORM, PracticeResult, PracticedWordEntryResult,
};
use crate::utils::shuffle::shuffle_strings;

const NONE_FIELD_KEY: &str = "none";

const PROMPT_FIELD_OPTIONS: [(&str, &str); 18] = [
    (NONE_FIELD_KEY, "无"),
    ("part_of_speech", "part_of_speech"),
    ("tags", "tags"),
    ("english", "english"),
    ("chinese", "chinese"),
    ("base_form", "base_form"),
    ("past_tense", "past_tense"),
    ("imperative", "imperative"),
    ("plural", "plural"),
    ("singular_definite", "singular_definite"),
    ("plural_definite", "plural_definite"),
    ("neuter_form", "neuter_form"),
    ("plural_form", "plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    ("adjective_superlative_definite", "adjective_superlative_definite"),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
];

const ANSWER_FIELD_OPTIONS: [(&str, &str); 15] = [
    ("english", "english"),
    ("chinese", "chinese"),
    ("base_form", "base_form"),
    ("past_tense", "past_tense"),
    ("imperative", "imperative"),
    ("plural", "plural"),
    ("singular_definite", "singular_definite"),
    ("plural_definite", "plural_definite"),
    ("neuter_form", "neuter_form"),
    ("plural_form", "plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    ("adjective_superlative_definite", "adjective_superlative_definite"),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
];

#[component]
pub fn LexiconPracticePage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();

    let selected_ids_on_enter = word_bank_state.selected_word_entry_ids.get_untracked();
    let base_practice_result = word_bank_state.practice_result.get_untracked();
    word_bank_state
        .set_temp_practice_result
        .set(create_temp_practice_result(
            &base_practice_result,
            &selected_ids_on_enter,
        ));

    let (questions_per_page, set_questions_per_page) = signal(10_usize);
    let (prompt_field_a, set_prompt_field_a) = signal("chinese".to_string());
    let (prompt_field_b, set_prompt_field_b) = signal("english".to_string());
    let (answer_fields, set_answer_fields) = signal(vec!["base_form".to_string()]);
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (active_question_ids, set_active_question_ids) = signal(Vec::<String>::new());
    let (solved_question_ids, set_solved_question_ids) = signal(Vec::<String>::new());

    Effect::new(move |_| {
        let selected_ids = word_bank_state.selected_word_entry_ids.get();
        let solved_ids = solved_question_ids.get();
        let page_size = questions_per_page.get().max(1);
        let current_active = active_question_ids.get_untracked();
        let next_active =
            refill_active_question_ids(&selected_ids, &current_active, &solved_ids, page_size);
        if next_active != current_active {
            set_active_question_ids.set(next_active);
        }
    });

    let check_click = move |_| {
        let selected_answer_fields = answer_fields.get_untracked();
        if selected_answer_fields.is_empty() {
            set_status.set("请先勾选至少 1 个“回答”项。".to_string());
            return;
        }

        let active_ids = active_question_ids.get_untracked();
        let entries = word_bank_state.entries.get_untracked();
        let current_questions = build_question_items(&active_ids, &entries);
        if current_questions.is_empty() {
            set_status.set("当前没有可检查的题目。".to_string());
            return;
        }

        let answers = answer_inputs.get_untracked();
        let mut field_results = Vec::<(String, String, bool, String)>::new();
        let mut newly_solved_ids = Vec::<String>::new();
        let mut total_fields = 0_usize;
        let mut correct_fields = 0_usize;

        for (_, entry) in &current_questions {
            let mut all_correct_for_entry = true;
            for field in &selected_answer_fields {
                let expected = entry_field_value(entry, field);
                let key = answer_input_key(&entry.id, field);
                let actual = answers.get(&key).cloned().unwrap_or_default();
                let is_correct = normalize_for_compare(&actual) == normalize_for_compare(&expected);

                total_fields += 1;
                if is_correct {
                    correct_fields += 1;
                } else {
                    all_correct_for_entry = false;
                }

                field_results.push((entry.id.clone(), field.clone(), is_correct, actual));
            }

            if all_correct_for_entry {
                newly_solved_ids.push(entry.id.clone());
            }
        }

        word_bank_state.set_temp_practice_result.update(|temp_result| {
            for (entry_id, field, is_correct, actual) in &field_results {
                record_field_check_result(temp_result, entry_id, field, *is_correct, actual);
            }
        });

        if !newly_solved_ids.is_empty() {
            let solved_set = newly_solved_ids
                .iter()
                .cloned()
                .collect::<HashSet<String>>();

            set_solved_question_ids.update(|solved_ids| {
                for entry_id in &newly_solved_ids {
                    if !solved_ids.iter().any(|existing| existing == entry_id) {
                        solved_ids.push(entry_id.clone());
                    }
                }
            });

            set_answer_inputs.update(|inputs| {
                inputs.retain(|key, _| {
                    !solved_set
                        .iter()
                        .any(|entry_id| key.starts_with(&format!("{entry_id}::")))
                });
            });
        }

        if newly_solved_ids.is_empty() {
            set_status.set(format!(
                "检查完成：字段正确 {correct_fields}/{total_fields}，暂无整题通过。"
            ));
        } else {
            set_status.set(format!(
                "检查完成：字段正确 {correct_fields}/{total_fields}，本轮完成 {} 条，已自动补充新题。",
                newly_solved_ids.len()
            ));
        }
    };

    let finish_click = move |_| {
        let mut selected_ids = word_bank_state.selected_word_entry_ids.get_untracked();
        if selected_ids.is_empty() {
            set_status.set("当前没有可练习词条。".to_string());
            return;
        }

        let temp_snapshot = word_bank_state.temp_practice_result.get_untracked();
        word_bank_state.set_practice_result.update(|global_result| {
            merge_practice_result(global_result, &temp_snapshot);
        });
        word_bank_state
            .set_last_completed_practice_result
            .set(temp_snapshot.clone());

        shuffle_strings(&mut selected_ids);
        word_bank_state.set_selected_word_entry_ids.set(selected_ids.clone());

        let latest_global = word_bank_state.practice_result.get_untracked();
        word_bank_state
            .set_temp_practice_result
            .set(create_temp_practice_result(&latest_global, &selected_ids));
        set_active_question_ids.set(Vec::new());
        set_solved_question_ids.set(Vec::new());
        set_answer_inputs.set(HashMap::new());
        set_current_page.set(AppPage::LexiconSummary);
    };

    let restart_click = move |_| {
        set_solved_question_ids.set(Vec::new());
        set_answer_inputs.set(HashMap::new());
        set_active_question_ids.set(Vec::new());
        set_status.set("已重新开始本轮练习（临时记录继续累加）。".to_string());
    };

    let abort_click = move |_| {
        word_bank_state
            .set_temp_practice_result
            .set(PracticeResult::default());
        set_current_page.set(AppPage::LexiconMode);
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <header class="mb-6 flex items-center justify-between gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"词库练习"</h1>
                    <button
                        type="button"
                        on:click=move |_| set_current_page.set(AppPage::LexiconMode)
                        class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm hover:bg-slate-700"
                    >
                        "返回选词页面"
                    </button>
                </header>

                <p class="mb-2 text-sm text-slate-300">
                    {move || {
                        let selected_count = word_bank_state.selected_word_entry_ids.get().len();
                        let temp_entries = word_bank_state.temp_practice_result.get().practiced_word_entries.len();
                        format!(
                            "当前可练习词条（selected=true）：{selected_count} 条，临时练习结果已记录 {temp_entries} 条。"
                        )
                    }}
                </p>
                <p class="mb-4 min-h-5 text-sm text-slate-300">{move || status.get()}</p>

                <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第一部分：练习设置"</h2>
                    <div class="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-3">
                        <label class="flex flex-col gap-1 text-sm">
                            <span class="text-slate-300">"一页练习（词条数）"</span>
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
                            <span class="text-slate-300">"根据（下拉框 1）"</span>
                            <select
                                prop:value=move || prompt_field_a.get()
                                on:change=move |ev| set_prompt_field_a.set(event_target_value(&ev))
                                class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                            >
                                {PROMPT_FIELD_OPTIONS
                                    .iter()
                                    .map(|(key, label)| view! { <option value=*key>{*label}</option> })
                                    .collect_view()}
                            </select>
                        </label>

                        <label class="flex flex-col gap-1 text-sm">
                            <span class="text-slate-300">"根据（下拉框 2）"</span>
                            <select
                                prop:value=move || prompt_field_b.get()
                                on:change=move |ev| set_prompt_field_b.set(event_target_value(&ev))
                                class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                            >
                                {PROMPT_FIELD_OPTIONS
                                    .iter()
                                    .map(|(key, label)| view! { <option value=*key>{*label}</option> })
                                    .collect_view()}
                            </select>
                        </label>
                    </div>

                    <div class="mt-4">
                        <p class="mb-2 text-sm text-slate-300">"回答（勾选要回答的项）"</p>
                        <div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-4">
                            {ANSWER_FIELD_OPTIONS
                                .iter()
                                .map(|(key, label)| {
                                    let key_for_checked = (*key).to_string();
                                    let key_for_change = (*key).to_string();
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
                                            <span>{*label}</span>
                                        </label>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第二部分：练习题"</h2>
                    <div class="mt-4 space-y-4">
                        <For
                            each=move || {
                                let active_ids = active_question_ids.get();
                                let entries = word_bank_state.entries.get();
                                build_question_items(&active_ids, &entries)
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

                    <div class="mt-4 flex justify-center">
                        <button
                            type="button"
                            on:click=check_click
                            class="inline-flex items-center rounded-lg border border-slate-700 bg-slate-800 px-6 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                        >
                            "检查"
                        </button>
                    </div>
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第三部分：流程控制"</h2>
                    <div class="mt-4 flex flex-wrap items-center gap-3">
                        <button
                            type="button"
                            on:click=finish_click
                            class="rounded-lg border border-emerald-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                        >
                            "完成练习"
                        </button>
                        <button
                            type="button"
                            on:click=restart_click
                            class="rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-700"
                        >
                            "重新练习"
                        </button>
                        <button
                            type="button"
                            on:click=abort_click
                            class="rounded-lg border border-red-800 bg-red-900/80 px-4 py-2 text-sm font-medium text-red-100 hover:bg-red-800"
                        >
                            "放弃练习并返回"
                        </button>
                    </div>
                </section>
            </section>
        </main>
    }
}

fn build_question_items(active_ids: &[String], entries: &[WordEntry]) -> Vec<(usize, WordEntry)> {
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

fn refill_active_question_ids(
    selected_ids: &[String],
    current_active_ids: &[String],
    solved_ids: &[String],
    page_size: usize,
) -> Vec<String> {
    let solved_set = solved_ids.iter().cloned().collect::<HashSet<String>>();
    let mut next_active = current_active_ids
        .iter()
        .filter(|id| !solved_set.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let mut used = next_active.iter().cloned().collect::<HashSet<String>>();

    for id in selected_ids {
        if next_active.len() >= page_size {
            break;
        }
        if solved_set.contains(id) || used.contains(id) {
            continue;
        }
        next_active.push(id.clone());
        used.insert(id.clone());
    }

    next_active.truncate(page_size.min(selected_ids.len()));
    next_active
}

fn create_temp_practice_result(base: &PracticeResult, selected_ids: &[String]) -> PracticeResult {
    PracticeResult {
        username: base.username.clone(),
        encryption_key: base.encryption_key.clone(),
        selected_word_entry_ids: selected_ids.to_vec(),
        practiced_word_entries: Vec::new(),
    }
}

fn answer_input_key(entry_id: &str, field: &str) -> String {
    format!("{entry_id}::{field}")
}

fn normalize_for_compare(value: &str) -> String {
    value
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("|")
        .to_lowercase()
}

fn record_field_check_result(
    temp_result: &mut PracticeResult,
    entry_id: &str,
    field: &str,
    is_correct: bool,
    user_input: &str,
) {
    let practiced_entry = get_or_insert_practiced_entry(temp_result, entry_id);
    let Some(stats) = answer_stats_mut(practiced_entry, field) else {
        return;
    };

    if is_correct {
        stats.correct_count += 1;
        return;
    }

    stats.wrong_count += 1;
    let wrong_input = user_input.trim();
    if wrong_input.is_empty() {
        return;
    }
    push_or_promote_wrong_answer(&mut stats.wrong_answers, wrong_input);
}

fn merge_practice_result(global_result: &mut PracticeResult, temp_result: &PracticeResult) {
    if global_result.username.trim().is_empty() && !temp_result.username.trim().is_empty() {
        global_result.username = temp_result.username.clone();
    }
    if global_result.encryption_key.trim().is_empty() && !temp_result.encryption_key.trim().is_empty()
    {
        global_result.encryption_key = temp_result.encryption_key.clone();
    }

    for id in &temp_result.selected_word_entry_ids {
        if !global_result
            .selected_word_entry_ids
            .iter()
            .any(|existing| existing == id)
        {
            global_result.selected_word_entry_ids.push(id.clone());
        }
    }

    for temp_entry in &temp_result.practiced_word_entries {
        if let Some(existing_entry) = global_result
            .practiced_word_entries
            .iter_mut()
            .find(|entry| entry.id == temp_entry.id)
        {
            merge_practiced_entry(existing_entry, temp_entry);
        } else {
            global_result.practiced_word_entries.push(temp_entry.clone());
        }
    }
}

fn merge_practiced_entry(
    existing_entry: &mut PracticedWordEntryResult,
    temp_entry: &PracticedWordEntryResult,
) {
    merge_answer_stats(&mut existing_entry.english, &temp_entry.english);
    merge_answer_stats(&mut existing_entry.chinese, &temp_entry.chinese);
    merge_answer_stats(&mut existing_entry.base_form, &temp_entry.base_form);
    merge_answer_stats(&mut existing_entry.past_tense, &temp_entry.past_tense);
    merge_answer_stats(&mut existing_entry.imperative, &temp_entry.imperative);
    merge_answer_stats(&mut existing_entry.plural, &temp_entry.plural);
    merge_answer_stats(
        &mut existing_entry.singular_definite,
        &temp_entry.singular_definite,
    );
    merge_answer_stats(&mut existing_entry.plural_definite, &temp_entry.plural_definite);
    merge_answer_stats(&mut existing_entry.neuter_form, &temp_entry.neuter_form);
    merge_answer_stats(&mut existing_entry.plural_form, &temp_entry.plural_form);
    merge_answer_stats(
        &mut existing_entry.adjective_comparative,
        &temp_entry.adjective_comparative,
    );
    merge_answer_stats(
        &mut existing_entry.adjective_superlative_indefinite,
        &temp_entry.adjective_superlative_indefinite,
    );
    merge_answer_stats(
        &mut existing_entry.adjective_superlative_definite,
        &temp_entry.adjective_superlative_definite,
    );
    merge_answer_stats(
        &mut existing_entry.adverb_comparative,
        &temp_entry.adverb_comparative,
    );
    merge_answer_stats(&mut existing_entry.adverb_superlative, &temp_entry.adverb_superlative);
}

fn merge_answer_stats(existing_stats: &mut AnswerStats, temp_stats: &AnswerStats) {
    existing_stats.correct_count += temp_stats.correct_count;
    existing_stats.wrong_count += temp_stats.wrong_count;

    // temp 中错法列表是“新到旧”，合并时从旧到新处理，保持新增顺序。
    for wrong in temp_stats.wrong_answers.iter().rev() {
        push_or_promote_wrong_answer(&mut existing_stats.wrong_answers, wrong);
    }
}

fn push_or_promote_wrong_answer(list: &mut Vec<String>, wrong_answer: &str) {
    let normalized = normalize_for_compare(wrong_answer);
    if normalized.is_empty() {
        return;
    }

    if let Some(existing_pos) = list
        .iter()
        .position(|current| normalize_for_compare(current) == normalized)
    {
        list.remove(existing_pos);
    }

    // 新错误（或重复错误）都放到最前面，表示最近一次出现。
    list.insert(0, wrong_answer.trim().to_string());
    if list.len() > MAX_WRONG_ANSWERS_PER_FORM {
        // 超出容量时删除最早插入（最旧）的那条。
        list.truncate(MAX_WRONG_ANSWERS_PER_FORM);
    }
}

fn get_or_insert_practiced_entry<'a>(
    temp_result: &'a mut PracticeResult,
    entry_id: &str,
) -> &'a mut PracticedWordEntryResult {
    if let Some(index) = temp_result
        .practiced_word_entries
        .iter()
        .position(|entry| entry.id == entry_id)
    {
        return &mut temp_result.practiced_word_entries[index];
    }

    let entry = PracticedWordEntryResult {
        id: entry_id.to_string(),
        ..PracticedWordEntryResult::default()
    };
    temp_result.practiced_word_entries.push(entry);
    let last_index = temp_result.practiced_word_entries.len() - 1;
    &mut temp_result.practiced_word_entries[last_index]
}

fn answer_stats_mut<'a>(
    practiced_entry: &'a mut PracticedWordEntryResult,
    field: &str,
) -> Option<&'a mut AnswerStats> {
    match field {
        "english" => Some(&mut practiced_entry.english),
        "chinese" => Some(&mut practiced_entry.chinese),
        "base_form" => Some(&mut practiced_entry.base_form),
        "past_tense" => Some(&mut practiced_entry.past_tense),
        "imperative" => Some(&mut practiced_entry.imperative),
        "plural" => Some(&mut practiced_entry.plural),
        "singular_definite" => Some(&mut practiced_entry.singular_definite),
        "plural_definite" => Some(&mut practiced_entry.plural_definite),
        "neuter_form" => Some(&mut practiced_entry.neuter_form),
        "plural_form" => Some(&mut practiced_entry.plural_form),
        "adjective_comparative" => Some(&mut practiced_entry.adjective_comparative),
        "adjective_superlative_indefinite" => {
            Some(&mut practiced_entry.adjective_superlative_indefinite)
        }
        "adjective_superlative_definite" => Some(&mut practiced_entry.adjective_superlative_definite),
        "adverb_comparative" => Some(&mut practiced_entry.adverb_comparative),
        "adverb_superlative" => Some(&mut practiced_entry.adverb_superlative),
        _ => None,
    }
}

fn entry_field_value(entry: &WordEntry, field: &str) -> String {
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
        "adjective_superlative_indefinite" => {
            entry.adjective_superlative_indefinite.clone().unwrap_or_default()
        }
        "adjective_superlative_definite" => {
            entry.adjective_superlative_definite.clone().unwrap_or_default()
        }
        "adverb_comparative" => entry.adverb_comparative.clone().unwrap_or_default(),
        "adverb_superlative" => entry.adverb_superlative.clone().unwrap_or_default(),
        _ => String::new(),
    }
}
