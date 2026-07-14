//! 月份系列练习页：使用固定题型（中文/英文提示 -> base_form 作答）进行练习。

use std::collections::HashMap;

use leptos::prelude::*;

use crate::app_state::{LexiconState, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::components::practice_engine::{
    AbortPracticeButton, CheckAnswersError, FinishPracticeButton, PracticeEntry,
    RestartPracticeButton, RestartTempBehavior, apply_check_results,
    clear_practice_round_local_state, evaluate_check_answers, merge_solved_question_ids,
    refill_active_question_ids, retain_unsolved_answer_inputs,
};
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::utils::i18n::tr;

use super::initialize_temp_practice_result;

#[component]
pub fn MonthSerisePracticePage() -> impl IntoView {
    const QUESTIONS_PER_PAGE: usize = 20;

    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;
    initialize_temp_practice_result(lexicon_state, practice_state);

    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let (prompt_field_a, _) = signal("chinese".to_string());
    let (prompt_field_b, _) = signal("english".to_string());
    let (answer_fields, _) = signal(vec!["base_form".to_string()]);
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (active_question_ids, set_active_question_ids) = signal(Vec::<String>::new());
    let (solved_question_ids, set_solved_question_ids) = signal(Vec::<String>::new());

    Effect::new(move |_| {
        let selected_ids = practice_state.selected_word_entry_ids.get();
        let solved_ids = solved_question_ids.get();
        let current_active = active_question_ids.get_untracked();
        let next_active = refill_active_question_ids(
            &selected_ids,
            &current_active,
            &solved_ids,
            QUESTIONS_PER_PAGE,
        );
        if next_active != current_active {
            set_active_question_ids.set(next_active);
        }
    });

    let check_click = Callback::new(move |_| {
        let language = lang.get_untracked();
        let selected_answer_fields = answer_fields.get_untracked();
        let active_ids = active_question_ids.get_untracked();
        let entries = lexicon_state.entries.get_untracked();
        let answers = answer_inputs.get_untracked();
        let check_result = match evaluate_check_answers(
            &selected_answer_fields,
            &active_ids,
            &entries,
            &answers,
        ) {
            Ok(result) => result,
            Err(CheckAnswersError::NoAnswerFields) => {
                set_status.set(
                    tr(
                        language,
                        "请先勾选至少 1 个“回答”项。",
                        "Please select at least one answer field.",
                    )
                    .to_string(),
                );
                return;
            }
            Err(CheckAnswersError::NoQuestions) => {
                set_status.set(
                    tr(
                        language,
                        "当前没有可检查的题目。",
                        "No questions available for checking.",
                    )
                    .to_string(),
                );
                return;
            }
            Err(CheckAnswersError::NoAnswerableFields) => {
                set_status.set(
                    tr(
                        language,
                        "当前题目在已选回答项下没有可作答字段。",
                        "No answerable fields under current answer settings.",
                    )
                    .to_string(),
                );
                return;
            }
        };

        practice_state
            .set_temp_practice_result
            .update(|temp_result| {
                apply_check_results(temp_result, &check_result.field_results);
            });

        if !check_result.newly_solved_ids.is_empty() {
            set_solved_question_ids.update(|solved_ids| {
                merge_solved_question_ids(solved_ids, &check_result.newly_solved_ids);
            });

            set_answer_inputs.update(|inputs| {
                retain_unsolved_answer_inputs(inputs, &check_result.newly_solved_ids);
            });
        }

        if check_result.newly_solved_ids.is_empty() {
            set_status.set(format!(
                "{} {}/{}，{}",
                tr(language, "检查完成：字段正确", "Checked: correct fields"),
                check_result.correct_fields,
                check_result.total_fields,
                tr(language, "暂无整题通过。", "no full entry solved yet.")
            ));
        } else {
            set_status.set(format!(
                "{} {}/{}，{} {} {}",
                tr(language, "检查完成：字段正确", "Checked: correct fields"),
                check_result.correct_fields,
                check_result.total_fields,
                tr(language, "本轮完成", "solved this round"),
                check_result.newly_solved_ids.len(),
                tr(
                    language,
                    "条，已自动补充新题。",
                    "entries, new questions appended."
                )
            ));
        }
    });

    let restart_ui_click = Callback::new(move |_| {
        clear_practice_round_local_state(
            set_solved_question_ids,
            set_answer_inputs,
            set_active_question_ids,
            None,
        );
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::SeriseSelect/>
                <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">
                    {move || tr(lang.get(), "月份系列练习", "Month Series Practice")}
                </h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let language = lang.get();
                        let selected_count = practice_state.selected_word_entry_ids.get().len();
                        let temp_entries = practice_state
                            .temp_practice_result
                            .get()
                            .practiced_word_entries
                            .len();
                        let overview = format!(
                            "{}：{selected_count} {}，{} {temp_entries} {}。",
                            tr(language, "当前可练习词条", "Available entries"),
                            tr(language, "条", "entries"),
                            tr(language, "临时练习结果已记录", "Temp result recorded"),
                            tr(language, "条", "entries")
                        );
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                tr(language, "等待作答并点击检查...", "Answer and click check...")
                                    .to_string()
                            } else {
                                s
                            }
                        };
                        format!("{overview}\n{status_line}")
                    })
                />

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第一部分：练习题", "Part 1: Questions")}
                    </h2>
                    <PracticeEntry
                        on_check=check_click
                        active_question_ids=active_question_ids
                        entries=lexicon_state.entries
                        prompt_field_a=prompt_field_a
                        prompt_field_b=prompt_field_b
                        answer_fields=answer_fields
                        answer_inputs=answer_inputs
                        set_answer_inputs=set_answer_inputs
                        allow_answer_reveal=None
                        revealed_answer_keys=None
                        set_revealed_answer_keys=None
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第二部分：流程控制", "Part 2: Flow Control")}
                    </h2>
                    <div class="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        <FinishPracticeButton
                            practice_state=practice_state
                            set_current_page=set_current_page
                            set_status=set_status
                            finish_target_page=AppPage::LexiconSummary
                            summary_return_page=AppPage::SeriseMonthPractice
                        />
                        <RestartPracticeButton
                            practice_state=practice_state
                            set_status=set_status
                            on_restart_ui=restart_ui_click
                            restart_message=tr(
                                lang.get_untracked(),
                                "已重新开始本轮练习（临时记录继续累加）。",
                                "Restarted this round (temp result keeps accumulating).",
                            )
                            .to_string()
                            restart_temp_behavior=RestartTempBehavior::Keep
                        />
                        <AbortPracticeButton
                            practice_state=practice_state
                            set_current_page=set_current_page
                            abort_target_page=AppPage::SeriseSelect
                        />
                    </div>
                </section>
            </section>
        </main>
    }
}
