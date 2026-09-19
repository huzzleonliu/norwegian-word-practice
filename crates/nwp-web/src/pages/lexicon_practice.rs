//! 词库练习页：承载题目生成、作答校验、临时统计与流程控制按钮。

use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::components::practice_engine::{
    AbortPracticeButton, CheckAnswersError, FinishPracticeButton, PracticeEntry, PracticeSettings,
    RestartPracticeButton, RestartTempBehavior, apply_check_results,
    clear_practice_round_local_state, create_temp_practice_result, default_answer_fields,
    evaluate_check_answers, merge_solved_question_ids, prepare_post_check_input_state,
    refill_active_question_ids, retain_unsolved_revealed_keys,
};
use crate::layout::{PageHeading, PageShell, PageTitle, PageTopbar, ShellAttach};
use crate::pages::AppPage;
use crate::utils::i18n::tr;

#[component]
pub fn LexiconPracticePage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;

    let selected_ids_on_enter = practice_state.selected_word_entry_ids.get_untracked();
    let base_practice_result = practice_state.practice_result.get_untracked();
    // 进入练习页时重建本轮临时结果；累计结果保存在 `practice_result`。
    practice_state
        .set_temp_practice_result
        .set(create_temp_practice_result(
            &base_practice_result,
            &selected_ids_on_enter,
        ));

    let (questions_per_page, set_questions_per_page) = signal(10_usize);
    let (prompt_field_a, set_prompt_field_a) = signal("chinese".to_string());
    let (prompt_field_b, set_prompt_field_b) = signal("english".to_string());
    let (prompt_field_c, set_prompt_field_c) = signal("part_of_speech".to_string());
    let (answer_fields, set_answer_fields) = signal(default_answer_fields());
    let (allow_answer_reveal, set_allow_answer_reveal) = signal(true);
    let (revealed_answer_keys, set_revealed_answer_keys) = signal(HashSet::<String>::new());
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (wrong_answer_feedback, set_wrong_answer_feedback) =
        signal(HashMap::<String, String>::new());
    let (active_question_ids, set_active_question_ids) = signal(Vec::<String>::new());
    let (solved_question_ids, set_solved_question_ids) = signal(Vec::<String>::new());

    Effect::new(move |_| {
        let selected_ids = practice_state.selected_word_entry_ids.get();
        let solved_ids = solved_question_ids.get();
        let page_size = questions_per_page.get().max(1);
        let current_active = active_question_ids.get_untracked();
        let next_active =
            refill_active_question_ids(&selected_ids, &current_active, &solved_ids, page_size);
        if next_active != current_active {
            set_active_question_ids.set(next_active);
        }
    });

    // 核心判题入口：逐字段比较 -> 写入临时统计 -> 全对题目移出当前页。
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

        let (cleared_inputs, wrong_feedback) = prepare_post_check_input_state(
            &check_result.field_results,
            &check_result.newly_solved_ids,
        );
        set_answer_inputs.set(cleared_inputs);
        set_wrong_answer_feedback.set(wrong_feedback);

        if !check_result.newly_solved_ids.is_empty() {
            set_solved_question_ids.update(|solved_ids| {
                merge_solved_question_ids(solved_ids, &check_result.newly_solved_ids);
            });

            set_revealed_answer_keys.update(|keys| {
                retain_unsolved_revealed_keys(keys, &check_result.newly_solved_ids);
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
            Some(set_revealed_answer_keys),
        );
        set_wrong_answer_feedback.set(HashMap::new());
    });

    let hide_all_revealed_answers = Callback::new(move |_| {
        set_revealed_answer_keys.set(HashSet::new());
    });

    view! {
        <PageShell attach=ShellAttach::Content>
            <PageTopbar>
                <PageHeading>
                    <PageTitle>
                        {move || tr(lang.get(), "词库练习", "Lexicon Practice")}
                    </PageTitle>
                </PageHeading>
            </PageTopbar>

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
                            "{}（selected=true）：{selected_count} {}，{} {temp_entries} {}。",
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

                <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第一部分：练习设置", "Part 1: Practice Settings")}
                    </h2>
                    <PracticeSettings
                        questions_per_page=questions_per_page
                        set_questions_per_page=set_questions_per_page
                        prompt_field_a=prompt_field_a
                        set_prompt_field_a=set_prompt_field_a
                        prompt_field_b=prompt_field_b
                        set_prompt_field_b=set_prompt_field_b
                        prompt_field_c=prompt_field_c
                        set_prompt_field_c=set_prompt_field_c
                        answer_fields=answer_fields
                        set_answer_fields=set_answer_fields
                        allow_answer_reveal=allow_answer_reveal
                        set_allow_answer_reveal=set_allow_answer_reveal
                        hide_all_revealed_answers=hide_all_revealed_answers
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第二部分：练习题", "Part 2: Questions")}
                    </h2>
                    <PracticeEntry
                        on_check=check_click
                        active_question_ids=active_question_ids
                        entries=lexicon_state.entries
                        prompt_field_a=prompt_field_a
                        prompt_field_b=prompt_field_b
                        prompt_field_c=prompt_field_c
                        answer_fields=answer_fields
                        answer_inputs=answer_inputs
                        set_answer_inputs=set_answer_inputs
                        allow_answer_reveal=Some(allow_answer_reveal)
                        revealed_answer_keys=Some(revealed_answer_keys)
                        set_revealed_answer_keys=Some(set_revealed_answer_keys)
                        wrong_answer_feedback=Some(wrong_answer_feedback)
                        set_wrong_answer_feedback=Some(set_wrong_answer_feedback)
                    />
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第三部分：流程控制", "Part 3: Flow Control")}
                    </h2>
                    <div class="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        <FinishPracticeButton
                            practice_state=practice_state
                            set_current_page=set_current_page
                            set_status=set_status
                            finish_target_page=AppPage::LexiconSummary
                            summary_return_page=AppPage::LexiconPractice
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
                            abort_target_page=AppPage::LexiconMode
                        />
                    </div>
                </section>
        </PageShell>
    }
}
