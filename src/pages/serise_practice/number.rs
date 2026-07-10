use std::collections::HashMap;

use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::WordEntry;
use crate::components::mini_console::MiniConsole;
use crate::components::practice_buttons::{
    RestartTempBehavior, handle_abort_click, handle_finish_click, handle_restart_click,
    normalize_for_compare, record_field_check_result,
};
use crate::components::practice_entry::answer_input_key;
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;

use super::initialize_temp_practice_result;

#[derive(Clone, Copy)]
struct NumberGroupConfig {
    key: &'static str,
    title: &'static str,
    hint: &'static str,
    numbers: &'static [u32],
}

#[derive(Clone)]
struct NumberQuestionRow {
    number: u32,
    cardinal_entry_id: String,
    ordinal_entry_id: String,
    cardinal_expected: String,
    ordinal_expected: String,
}

const GROUP_1_NUMBERS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
const GROUP_2_NUMBERS: &[u32] = &[21, 22, 23, 24, 25, 26, 27, 28, 29, 30];
const GROUP_3_NUMBERS: &[u32] = &[
    40, 50, 60, 70, 80, 90, 100, 200, 300, 500, 1000, 5000, 100000,
];
const NUMBER_GROUPS: [NumberGroupConfig; 3] = [
    NumberGroupConfig {
        key: "group-1-20",
        title: "第一块：1-20",
        hint: "每个数字填写两个词：左侧基数词，右侧序数词。",
        numbers: GROUP_1_NUMBERS,
    },
    NumberGroupConfig {
        key: "group-21-30",
        title: "第二块：21-30",
        hint: "继续按数字顺序填写基数词与序数词。",
        numbers: GROUP_2_NUMBERS,
    },
    NumberGroupConfig {
        key: "group-selected-large",
        title: "第三块：40-100000（精选）",
        hint: "本块按 readme 精选数字出题，不是连续每个数字都出题。",
        numbers: GROUP_3_NUMBERS,
    },
];

#[component]
pub fn NumberSerisePracticePage() -> impl IntoView {
    let word_bank_state = expect_context::<WordBankState>();
    initialize_temp_practice_result(word_bank_state);

    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (group_statuses, set_group_statuses) = signal(HashMap::<String, String>::new());

    let restart_ui_click = Callback::new(move |_| {
        set_answer_inputs.set(HashMap::new());
        set_group_statuses.set(HashMap::new());
    });

    let finish_click = move |_| {
        handle_finish_click(
            word_bank_state,
            set_current_page,
            set_status,
            AppPage::LexiconSummary,
        );
    };

    let restart_click = move |_| {
        handle_restart_click(
            word_bank_state,
            set_status,
            restart_ui_click,
            "已重新开始本轮练习（临时记录继续累加）。",
            RestartTempBehavior::Keep,
        );
    };

    let abort_click = move |_| {
        handle_abort_click(word_bank_state, set_current_page, AppPage::SeriseSelect);
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <ReturnButton target_page=AppPage::SeriseSelect/>
                <h1 class="text-3xl font-bold tracking-tight">"数词系列练习"</h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let selected_count = word_bank_state.selected_word_entry_ids.get().len();
                        let temp_entries = word_bank_state
                            .temp_practice_result
                            .get()
                            .practiced_word_entries
                            .len();
                        let overview = format!(
                            "当前可练习词条：{selected_count} 条，临时练习结果已记录 {temp_entries} 条。"
                        );
                        let status_line = {
                            let s = status.get();
                            if s.trim().is_empty() {
                                "等待分组作答并点击对应分组检查按钮...".to_string()
                            } else {
                                s
                            }
                        };
                        format!("{overview}\n{status_line}")
                    })
                />

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"请写出以下数字的基数词和序数词"</h2>
                    <p class="mt-2 text-sm text-slate-400">
                        "每行左侧输入基数词，右侧输入序数词。每个输入框都会映射到对应词条的 base_form。"
                    </p>
                </section>

                {NUMBER_GROUPS
                    .iter()
                    .map(|group| {
                        let group_key_for_check = group.key.to_string();
                        let group_key_for_msg = group.key.to_string();
                        let numbers = group.numbers;
                        let group_title = group.title;
                        let group_hint = group.hint;

                        let check_group_click = move |_| {
                            let entries = word_bank_state.entries.get_untracked();
                            let (rows, missing_numbers) = build_number_question_rows(&entries, numbers);
                            if rows.is_empty() {
                                let no_data_message = "当前分组没有可检查题目，请先确认系列词库加载正常。".to_string();
                                set_group_statuses.update(|messages| {
                                    messages.insert(group_key_for_check.clone(), no_data_message.clone());
                                });
                                set_status.set(no_data_message);
                                return;
                            }

                            let answers = answer_inputs.get_untracked();
                            let mut total_fields = 0_usize;
                            let mut correct_fields = 0_usize;
                            let mut fully_correct_rows = 0_usize;
                            let mut field_results = Vec::<(String, bool, String)>::new();

                            for row in &rows {
                                let cardinal_key = answer_input_key(&row.cardinal_entry_id, "base_form");
                                let ordinal_key = answer_input_key(&row.ordinal_entry_id, "base_form");
                                let cardinal_actual = answers.get(&cardinal_key).cloned().unwrap_or_default();
                                let ordinal_actual = answers.get(&ordinal_key).cloned().unwrap_or_default();

                                let cardinal_correct = normalize_for_compare(&cardinal_actual)
                                    == normalize_for_compare(&row.cardinal_expected);
                                let ordinal_correct = normalize_for_compare(&ordinal_actual)
                                    == normalize_for_compare(&row.ordinal_expected);

                                total_fields += 2;
                                if cardinal_correct {
                                    correct_fields += 1;
                                }
                                if ordinal_correct {
                                    correct_fields += 1;
                                }
                                if cardinal_correct && ordinal_correct {
                                    fully_correct_rows += 1;
                                }

                                field_results.push((
                                    row.cardinal_entry_id.clone(),
                                    cardinal_correct,
                                    cardinal_actual,
                                ));
                                field_results.push((
                                    row.ordinal_entry_id.clone(),
                                    ordinal_correct,
                                    ordinal_actual,
                                ));
                            }

                            word_bank_state
                                .set_temp_practice_result
                                .update(|temp_result| {
                                    for (entry_id, is_correct, actual) in &field_results {
                                        record_field_check_result(
                                            temp_result,
                                            entry_id,
                                            "base_form",
                                            *is_correct,
                                            actual,
                                        );
                                    }
                                });

                            let base_message = format!(
                                "检查完成：字段正确 {correct_fields}/{total_fields}，整行全对 {fully_correct_rows}/{}。",
                                rows.len()
                            );
                            let final_message = if missing_numbers.is_empty() {
                                base_message
                            } else {
                                format!(
                                    "{} 缺失题号：{}。",
                                    base_message,
                                    missing_numbers
                                        .iter()
                                        .map(|n| n.to_string())
                                        .collect::<Vec<_>>()
                                        .join("、")
                                )
                            };
                            set_group_statuses.update(|messages| {
                                messages.insert(group_key_for_check.clone(), final_message.clone());
                            });
                            set_status.set(format!("{group_title}：{final_message}"));
                        };

                        view! {
                            <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                                <div class="flex flex-wrap items-center justify-between gap-3">
                                    <h3 class="text-lg font-semibold">{group_title}</h3>
                                    <button
                                        type="button"
                                        on:click=check_group_click
                                        class="rounded-lg border border-emerald-600 bg-emerald-700 px-4 py-2 text-sm font-semibold text-white hover:bg-emerald-600"
                                    >
                                        "检查本组"
                                    </button>
                                </div>
                                <p class="mt-2 text-sm text-slate-400">{group_hint}</p>

                                <div class="mt-4 space-y-3">
                                    {move || {
                                        let entries = word_bank_state.entries.get();
                                        let (rows, missing_numbers) = build_number_question_rows(&entries, numbers);
                                        if rows.is_empty() {
                                            return view! {
                                                <p class="text-sm text-amber-300">
                                                    "当前分组无可用题目。请检查词库数据是否完整。"
                                                </p>
                                            }
                                                .into_any();
                                        }

                                        let rows_view = rows
                                            .into_iter()
                                            .map(|row| {
                                                let cardinal_key_for_value = answer_input_key(
                                                    &row.cardinal_entry_id,
                                                    "base_form",
                                                );
                                                let cardinal_key_for_input = cardinal_key_for_value.clone();
                                                let ordinal_key_for_value = answer_input_key(
                                                    &row.ordinal_entry_id,
                                                    "base_form",
                                                );
                                                let ordinal_key_for_input = ordinal_key_for_value.clone();

                                                view! {
                                                    <article class="grid grid-cols-1 gap-3 rounded-lg border border-slate-800 bg-slate-900/40 p-3 md:grid-cols-[90px_1fr_1fr] md:items-center">
                                                        <div class="text-sm font-semibold text-slate-200">
                                                            {format!("数字 {}", row.number)}
                                                        </div>
                                                        <label class="flex flex-col gap-1 text-xs text-slate-300">
                                                            <span>"基数词（cardinal）"</span>
                                                            <input
                                                                type="text"
                                                                prop:value=move || {
                                                                    answer_inputs
                                                                        .get()
                                                                        .get(&cardinal_key_for_value)
                                                                        .cloned()
                                                                        .unwrap_or_default()
                                                                }
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    set_answer_inputs.update(|inputs| {
                                                                        inputs.insert(cardinal_key_for_input.clone(), value);
                                                                    });
                                                                }
                                                                placeholder="填写基数词"
                                                                class="rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                            />
                                                        </label>
                                                        <label class="flex flex-col gap-1 text-xs text-slate-300">
                                                            <span>"序数词（ordinal）"</span>
                                                            <input
                                                                type="text"
                                                                prop:value=move || {
                                                                    answer_inputs
                                                                        .get()
                                                                        .get(&ordinal_key_for_value)
                                                                        .cloned()
                                                                        .unwrap_or_default()
                                                                }
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    set_answer_inputs.update(|inputs| {
                                                                        inputs.insert(ordinal_key_for_input.clone(), value);
                                                                    });
                                                                }
                                                                placeholder="填写序数词"
                                                                class="rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                            />
                                                        </label>
                                                    </article>
                                                }
                                            })
                                            .collect_view();

                                        if missing_numbers.is_empty() {
                                            view! { {rows_view} }.into_any()
                                        } else {
                                            let missing_text = missing_numbers
                                                .iter()
                                                .map(|n| n.to_string())
                                                .collect::<Vec<_>>()
                                                .join("、");
                                            view! {
                                                <>
                                                    {rows_view}
                                                    <p class="text-xs text-amber-300">
                                                        {format!("提示：以下题号未找到完整基数词/序数词词条：{missing_text}")}
                                                    </p>
                                                </>
                                            }
                                                .into_any()
                                        }
                                    }}
                                </div>

                                <p class="mt-3 min-h-5 text-sm text-slate-300">
                                    {move || {
                                        group_statuses
                                            .get()
                                            .get(&group_key_for_msg)
                                            .cloned()
                                            .unwrap_or_else(|| "待检查".to_string())
                                    }}
                                </p>
                            </section>
                        }
                    })
                    .collect_view()}

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"流程控制"</h2>
                    <div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        <button
                            type="button"
                            on:click=finish_click
                            class="rounded-lg border border-indigo-600 bg-indigo-700 px-4 py-3 text-sm font-semibold text-white hover:bg-indigo-600"
                        >
                            "完成练习"
                        </button>
                        <button
                            type="button"
                            on:click=restart_click
                            class="rounded-lg border border-amber-600 bg-amber-700 px-4 py-3 text-sm font-semibold text-white hover:bg-amber-600"
                        >
                            "重新练习"
                        </button>
                        <button
                            type="button"
                            on:click=abort_click
                            class="rounded-lg border border-rose-600 bg-rose-700 px-4 py-3 text-sm font-semibold text-white hover:bg-rose-600"
                        >
                            "放弃练习并返回"
                        </button>
                    </div>
                </section>
            </section>
        </main>
    }
}

fn build_number_question_rows(
    entries: &[WordEntry],
    numbers: &[u32],
) -> (Vec<NumberQuestionRow>, Vec<u32>) {
    let mut rows = Vec::new();
    let mut missing_numbers = Vec::new();

    for number in numbers {
        let Some(cardinal_chinese) = number_to_cardinal_chinese(*number) else {
            missing_numbers.push(*number);
            continue;
        };
        let ordinal_chinese = format!("第{cardinal_chinese}");

        let Some(cardinal_entry) =
            find_entry_by_pos_and_chinese(entries, "cardinal_number", cardinal_chinese)
        else {
            missing_numbers.push(*number);
            continue;
        };
        let Some(ordinal_entry) =
            find_entry_by_pos_and_chinese(entries, "ordinal_number", &ordinal_chinese)
        else {
            missing_numbers.push(*number);
            continue;
        };

        rows.push(NumberQuestionRow {
            number: *number,
            cardinal_entry_id: cardinal_entry.id.clone(),
            ordinal_entry_id: ordinal_entry.id.clone(),
            cardinal_expected: cardinal_entry.base_form.clone(),
            ordinal_expected: ordinal_entry.base_form.clone(),
        });
    }

    (rows, missing_numbers)
}

fn find_entry_by_pos_and_chinese<'a>(
    entries: &'a [WordEntry],
    part_of_speech: &str,
    chinese: &str,
) -> Option<&'a WordEntry> {
    entries.iter().find(|entry| {
        entry.part_of_speech == part_of_speech
            && entry
                .chinese
                .iter()
                .any(|candidate| normalize_for_compare(candidate) == normalize_for_compare(chinese))
    })
}

fn number_to_cardinal_chinese(number: u32) -> Option<&'static str> {
    match number {
        1 => Some("一"),
        2 => Some("二"),
        3 => Some("三"),
        4 => Some("四"),
        5 => Some("五"),
        6 => Some("六"),
        7 => Some("七"),
        8 => Some("八"),
        9 => Some("九"),
        10 => Some("十"),
        11 => Some("十一"),
        12 => Some("十二"),
        13 => Some("十三"),
        14 => Some("十四"),
        15 => Some("十五"),
        16 => Some("十六"),
        17 => Some("十七"),
        18 => Some("十八"),
        19 => Some("十九"),
        20 => Some("二十"),
        21 => Some("二十一"),
        22 => Some("二十二"),
        23 => Some("二十三"),
        24 => Some("二十四"),
        25 => Some("二十五"),
        26 => Some("二十六"),
        27 => Some("二十七"),
        28 => Some("二十八"),
        29 => Some("二十九"),
        30 => Some("三十"),
        40 => Some("四十"),
        50 => Some("五十"),
        60 => Some("六十"),
        70 => Some("七十"),
        80 => Some("八十"),
        90 => Some("九十"),
        100 => Some("一百"),
        200 => Some("二百"),
        300 => Some("三百"),
        500 => Some("五百"),
        1000 => Some("一千"),
        5000 => Some("五千"),
        100000 => Some("十万"),
        _ => None,
    }
}
