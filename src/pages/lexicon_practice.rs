use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::mini_console::MiniConsole;
use crate::components::practice_buttons::{
    AbortPracticeButton, CheckPracticeButton, FinishPracticeButton, RestartPracticeButton,
    RestartTempBehavior, create_temp_practice_result, normalize_for_compare,
    record_field_check_result,
};
use crate::components::practice_entry::{
    PracticeEntry, answer_input_key, build_question_items, entry_field_value,
    is_answer_field_available,
};
use crate::components::practice_settings::{PracticeSettings, default_answer_fields};
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;

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
    let (answer_fields, set_answer_fields) = signal(default_answer_fields());
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

    let check_click = Callback::new(move |_| {
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
            let mut checked_any_field = false;
            for field in &selected_answer_fields {
                if !is_answer_field_available(entry, field) {
                    continue;
                }
                checked_any_field = true;

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

            if checked_any_field && all_correct_for_entry {
                newly_solved_ids.push(entry.id.clone());
            }
        }

        if total_fields == 0 {
            set_status.set("当前题目在已选回答项下没有可作答字段。".to_string());
            return;
        }

        word_bank_state
            .set_temp_practice_result
            .update(|temp_result| {
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
    });

    let restart_ui_click = Callback::new(move |_| {
        set_solved_question_ids.set(Vec::new());
        set_answer_inputs.set(HashMap::new());
        set_active_question_ids.set(Vec::new());
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <ReturnButton target_page=AppPage::LexiconMode/>
                <header class="mb-6 flex items-center gap-4">
                    <h1 class="text-2xl font-bold tracking-tight">"词库练习"</h1>
                </header>

                <MiniConsole
                    message=Signal::derive(move || {
                        let selected_count = word_bank_state.selected_word_entry_ids.get().len();
                        let temp_entries = word_bank_state.temp_practice_result.get().practiced_word_entries.len();
                        let overview = format!(
                            "当前可练习词条（selected=true）：{selected_count} 条，临时练习结果已记录 {temp_entries} 条。"
                        );
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                "等待作答并点击检查...".to_string()
                            } else {
                                s
                            }
                        };
                        format!("{overview}\n{status_line}")
                    })
                />

                <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第一部分：练习设置"</h2>
                    <PracticeSettings
                        questions_per_page=questions_per_page
                        set_questions_per_page=set_questions_per_page
                        prompt_field_a=prompt_field_a
                        set_prompt_field_a=set_prompt_field_a
                        prompt_field_b=prompt_field_b
                        set_prompt_field_b=set_prompt_field_b
                        answer_fields=answer_fields
                        set_answer_fields=set_answer_fields
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第二部分：练习题"</h2>
                    <PracticeEntry
                        active_question_ids=active_question_ids
                        entries=word_bank_state.entries
                        prompt_field_a=prompt_field_a
                        prompt_field_b=prompt_field_b
                        answer_fields=answer_fields
                        answer_inputs=answer_inputs
                        set_answer_inputs=set_answer_inputs
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"第三部分：流程控制"</h2>
                    <div class="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                        <CheckPracticeButton on_check=check_click/>
                        <FinishPracticeButton
                            word_bank_state=word_bank_state
                            set_current_page=set_current_page
                            set_status=set_status
                            finish_target_page=AppPage::LexiconSummary
                            summary_return_page=AppPage::LexiconPractice
                        />
                        <RestartPracticeButton
                            word_bank_state=word_bank_state
                            set_status=set_status
                            on_restart_ui=restart_ui_click
                            restart_message="已重新开始本轮练习（临时记录继续累加）。".to_string()
                            restart_temp_behavior=RestartTempBehavior::Keep
                        />
                        <AbortPracticeButton
                            word_bank_state=word_bank_state
                            set_current_page=set_current_page
                            abort_target_page=AppPage::LexiconMode
                        />
                    </div>
                </section>
            </section>
        </main>
    }
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
