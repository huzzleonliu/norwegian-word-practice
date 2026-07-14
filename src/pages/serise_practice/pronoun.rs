//! 代词系列练习页：基于“人称 × 形式”配置渲染题组并逐字段校验答案。

use std::collections::HashMap;

use leptos::prelude::*;

use crate::app_state::{LexiconState, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::components::practice_engine::{
    AbortPracticeButton, CheckPracticeButton, FinishPracticeButton, RestartPracticeButton,
    RestartTempBehavior, answer_input_key, normalize_for_compare, record_field_check_result,
};
use crate::components::return_button::ReturnButton;
use crate::pages::AppPage;
use crate::structures::word_bank_entry::WordBankEntry;
use crate::utils::i18n::tr;

use super::initialize_temp_practice_result;

#[derive(Clone, Copy)]
struct PronounFieldConfig {
    label: &'static str,
    chinese: &'static str,
}

#[derive(Clone, Copy)]
struct PronounRowConfig {
    prompt: &'static str,
    fields: &'static [PronounFieldConfig],
}

#[derive(Clone, Copy)]
struct PronounGroupConfig {
    key: &'static str,
    title: &'static str,
    hint: &'static str,
    rows: &'static [PronounRowConfig],
}

#[derive(Clone)]
struct ResolvedPronounField {
    label: String,
    entry_id: String,
    expected: String,
}

#[derive(Clone)]
struct ResolvedPronounRow {
    prompt: String,
    fields: Vec<ResolvedPronounField>,
}

const CASE_MY: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "我（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "我（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "我的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "我的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "我的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "我自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "我自己的（反身物主）",
    },
];
const CASE_YOUR_SG: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "你（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "你（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "你的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "你的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "你的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "你自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "你自己的（反身物主）",
    },
];
const CASE_HIS: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "他（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "他（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "他的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "他的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "他的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "他自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "他自己的（反身物主）",
    },
];
const CASE_HER: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "她（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "她（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "她的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "她的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "她的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "她自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "她自己的（反身物主）",
    },
];
const CASE_OUR: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "我们（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "我们（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "我们的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "我们的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "我们的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "我们自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "我们自己的（反身物主）",
    },
];
const CASE_YOUR_PL: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "你们（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "你们（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "你们的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "你们的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "你们的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "你们自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "你们自己的（反身物主）",
    },
];
const CASE_THEIR_MALE: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "他们（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "他们（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "他们的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "他们的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "他们的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "他们自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "他们自己的（反身物主）",
    },
];
const CASE_THEIR_FEMALE: &[PronounFieldConfig] = &[
    PronounFieldConfig {
        label: "主格",
        chinese: "她们（主格）",
    },
    PronounFieldConfig {
        label: "宾格",
        chinese: "她们（宾格）",
    },
    PronounFieldConfig {
        label: "所有格阴阳性",
        chinese: "她们的（阴阳性）",
    },
    PronounFieldConfig {
        label: "所有格中性",
        chinese: "她们的（中性）",
    },
    PronounFieldConfig {
        label: "所有格复数",
        chinese: "她们的（复数）",
    },
    PronounFieldConfig {
        label: "反身代词",
        chinese: "她们自己（反身）",
    },
    PronounFieldConfig {
        label: "反身物主代词",
        chinese: "她们自己的（反身物主）",
    },
];
const CASE_THIS: &[PronounFieldConfig] = &[PronounFieldConfig {
    label: "指示代词",
    chinese: "这",
}];
const CASE_THAT: &[PronounFieldConfig] = &[PronounFieldConfig {
    label: "指示代词",
    chinese: "那",
}];
const CASE_THESE: &[PronounFieldConfig] = &[PronounFieldConfig {
    label: "指示代词",
    chinese: "这些",
}];
const CASE_THOSE: &[PronounFieldConfig] = &[PronounFieldConfig {
    label: "指示代词",
    chinese: "那些",
}];

const GROUP_1_ROWS: &[PronounRowConfig] = &[
    PronounRowConfig {
        prompt: "我",
        fields: CASE_MY,
    },
    PronounRowConfig {
        prompt: "你",
        fields: CASE_YOUR_SG,
    },
    PronounRowConfig {
        prompt: "他",
        fields: CASE_HIS,
    },
    PronounRowConfig {
        prompt: "她",
        fields: CASE_HER,
    },
];
const GROUP_2_ROWS: &[PronounRowConfig] = &[
    PronounRowConfig {
        prompt: "我们",
        fields: CASE_OUR,
    },
    PronounRowConfig {
        prompt: "你们",
        fields: CASE_YOUR_PL,
    },
    PronounRowConfig {
        prompt: "他们",
        fields: CASE_THEIR_MALE,
    },
    PronounRowConfig {
        prompt: "她们",
        fields: CASE_THEIR_FEMALE,
    },
];
const GROUP_3_ROWS: &[PronounRowConfig] = &[
    PronounRowConfig {
        prompt: "这",
        fields: CASE_THIS,
    },
    PronounRowConfig {
        prompt: "那",
        fields: CASE_THAT,
    },
    PronounRowConfig {
        prompt: "这些",
        fields: CASE_THESE,
    },
    PronounRowConfig {
        prompt: "那些",
        fields: CASE_THOSE,
    },
];
const PRONOUN_GROUPS: [PronounGroupConfig; 3] = [
    PronounGroupConfig {
        key: "group-pronoun-1",
        title: "第一块：我 / 你 / 他 / 她",
        hint: "题面是人称，填写主格、宾格、所有格（阴阳/中性/复数）、反身、反身物主。",
        rows: GROUP_1_ROWS,
    },
    PronounGroupConfig {
        key: "group-pronoun-2",
        title: "第二块：我们 / 你们 / 他们 / 她们",
        hint: "同样填写 7 类形式；每行对应一个人称。",
        rows: GROUP_2_ROWS,
    },
    PronounGroupConfig {
        key: "group-pronoun-3",
        title: "第三块：这 / 那 / 这些 / 那些",
        hint: "指示代词按行填写对应原型。",
        rows: GROUP_3_ROWS,
    },
];

#[component]
pub fn PronounSerisePracticePage() -> impl IntoView {
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;
    initialize_temp_practice_result(lexicon_state, practice_state);

    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (group_statuses, set_group_statuses) = signal(HashMap::<String, String>::new());

    let restart_ui_click = Callback::new(move |_| {
        set_answer_inputs.set(HashMap::new());
        set_group_statuses.set(HashMap::new());
    });

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <ReturnButton target_page=AppPage::SeriseSelect/>
                <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">
                    {move || tr(lang.get(), "代词系列练习", "Pronoun Series Practice")}
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
                                tr(
                                    language,
                                    "等待分组作答并点击对应分组检查按钮...",
                                    "Answer by group and click group check button...",
                                )
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
                        {move || tr(lang.get(), "请写出以下代词的各种形式", "Write all forms for pronouns")}
                    </h2>
                    <p class="mt-2 text-sm text-slate-400">
                        {move || {
                            tr(
                                lang.get(),
                                "每个输入框都绑定到对应词条的 base_form，检查后会按词条结构化记录结果。",
                                "Each input maps to the entry base_form and records structured results after checking.",
                            )
                        }}
                    </p>
                </section>

                {PRONOUN_GROUPS
                    .iter()
                    .map(|group| {
                        let group_key_for_check = group.key.to_string();
                        let group_key_for_msg = group.key.to_string();
                        let group_title = group.title;
                        let group_hint = group.hint;
                        let group_rows = group.rows;

                        let check_group_click = Callback::new(move |_| {
                            let language = lang.get_untracked();
                            let entries = selected_series_entries(lexicon_state, practice_state);
                            let (rows, missing_fields) = build_pronoun_rows(&entries, group_rows);
                            if rows.is_empty() {
                                let message = tr(
                                    language,
                                    "当前分组没有可检查题目，请先确认系列词库加载正常。",
                                    "No checkable questions in this group. Please confirm series lexicon is loaded.",
                                )
                                .to_string();
                                set_group_statuses.update(|messages| {
                                    messages.insert(group_key_for_check.clone(), message.clone());
                                });
                                set_status.set(message);
                                return;
                            }

                            let answers = answer_inputs.get_untracked();
                            let mut total_fields = 0_usize;
                            let mut correct_fields = 0_usize;
                            let mut fully_correct_rows = 0_usize;
                            let mut field_results = Vec::<(String, bool, String)>::new();

                            for row in &rows {
                                let mut row_all_correct = true;
                                for field in &row.fields {
                                    let key = answer_input_key(&field.entry_id, "base_form");
                                    let actual = answers.get(&key).cloned().unwrap_or_default();
                                    let is_correct = normalize_for_compare(&actual)
                                        == normalize_for_compare(&field.expected);

                                    total_fields += 1;
                                    if is_correct {
                                        correct_fields += 1;
                                    } else {
                                        row_all_correct = false;
                                    }

                                    field_results.push((field.entry_id.clone(), is_correct, actual));
                                }
                                if row_all_correct && !row.fields.is_empty() {
                                    fully_correct_rows += 1;
                                }
                            }

                            practice_state
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
                                "{} {correct_fields}/{total_fields}，{} {fully_correct_rows}/{}。",
                                tr(language, "检查完成：字段正确", "Checked: correct fields"),
                                tr(language, "整行全对", "fully correct rows"),
                                rows.len()
                            );
                            let final_message = if missing_fields.is_empty() {
                                base_message
                            } else {
                                format!(
                                    "{} {}: {}.",
                                    base_message,
                                    tr(language, "缺失", "missing"),
                                    missing_fields.join(", ")
                                )
                            };
                            set_group_statuses.update(|messages| {
                                messages.insert(group_key_for_check.clone(), final_message.clone());
                            });
                            set_status.set(format!(
                                "{}: {final_message}",
                                tr(language, group_title, pronoun_group_title_en(group_title))
                            ));
                        });

                        view! {
                            <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                                <div class="flex flex-wrap items-center justify-between gap-3">
                                    <h3 class="text-lg font-semibold">
                                        {move || tr(lang.get(), group_title, pronoun_group_title_en(group_title))}
                                    </h3>
                                    <CheckPracticeButton
                                        on_check=check_group_click
                                        label=tr(lang.get_untracked(), "检查本组", "Check Group").to_string()
                                        class="w-full rounded-lg border border-emerald-600 bg-emerald-700 px-4 py-2 text-sm font-semibold text-white hover:bg-emerald-600 sm:w-auto".to_string()
                                    />
                                </div>
                                <p class="mt-2 text-sm text-slate-400">
                                    {move || tr(lang.get(), group_hint, pronoun_group_hint_en(group_hint))}
                                </p>

                                <div class="mt-4 space-y-3">
                                    {move || {
                                        let entries = selected_series_entries(lexicon_state, practice_state);
                                        let (rows, missing_fields) = build_pronoun_rows(&entries, group_rows);
                                        if rows.is_empty() {
                                            return view! {
                                                <p class="text-sm text-amber-300">
                                                    {move || {
                                                        tr(
                                                            lang.get(),
                                                            "当前分组无可用题目。请检查词库数据是否完整。",
                                                            "No questions in this group. Please check lexicon data.",
                                                        )
                                                    }}
                                                </p>
                                            }
                                                .into_any();
                                        }

                                        let rows_view = rows
                                            .into_iter()
                                            .map(|row| {
                                                let prompt = row.prompt.clone();
                                                let field_view = row
                                                    .fields
                                                    .into_iter()
                                                    .map(|field| {
                                                        let field_label = field.label.clone();
                                                        let key_for_value =
                                                            answer_input_key(&field.entry_id, "base_form");
                                                        let key_for_input = key_for_value.clone();
                                                        view! {
                                                            <label class="flex flex-col gap-1 text-xs text-slate-300">
                                                                <span>
                                                                    {move || {
                                                                        tr(
                                                                            lang.get(),
                                                                            &field_label,
                                                                            pronoun_field_label_en(&field_label),
                                                                        )
                                                                        .to_string()
                                                                    }}
                                                                </span>
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
                                                                    placeholder=move || tr(lang.get(), "填写该形式", "Type this form")
                                                                    class="rounded border border-slate-700 bg-slate-950 px-2 py-2 text-sm text-slate-100"
                                                                />
                                                            </label>
                                                        }
                                                    })
                                                    .collect_view();

                                                view! {
                                                    <article class="grid grid-cols-1 gap-3 rounded-lg border border-slate-800 bg-slate-900/40 p-3">
                                                        <div class="text-sm font-semibold text-slate-200">
                                                            {move || {
                                                                tr(lang.get(), &prompt, pronoun_prompt_en(&prompt))
                                                                    .to_string()
                                                            }}
                                                        </div>
                                                        <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
                                                            {field_view}
                                                        </div>
                                                    </article>
                                                }
                                            })
                                            .collect_view();

                                        if missing_fields.is_empty() {
                                            view! { {rows_view} }.into_any()
                                        } else {
                                            view! {
                                                <>
                                                    {rows_view}
                                                    <p class="text-xs text-amber-300">
                                                        {move || format!(
                                                            "{}{}",
                                                            tr(lang.get(), "提示：以下题目项未找到对应词条：", "Hint: missing mapped entries for: "),
                                                            missing_fields.join(", ")
                                                        )}
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
                                            .unwrap_or_else(|| tr(lang.get(), "待检查", "Pending").to_string())
                                    }}
                                </p>
                            </section>
                        }
                    })
                    .collect_view()}

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">{move || tr(lang.get(), "流程控制", "Flow Control")}</h2>
                    <div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        <FinishPracticeButton
                            practice_state=practice_state
                            set_current_page=set_current_page
                            set_status=set_status
                            finish_target_page=AppPage::LexiconSummary
                            summary_return_page=AppPage::SerisePronounPractice
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

fn pronoun_group_title_en(zh: &str) -> &'static str {
    match zh {
        "第一块：我 / 你 / 他 / 她" => "Group 1: I / You / He / She",
        "第二块：我们 / 你们 / 他们 / 她们" => {
            "Group 2: We / You(pl) / They(m) / They(f)"
        }
        "第三块：这 / 那 / 这些 / 那些" => "Group 3: This / That / These / Those",
        _ => "Pronoun Group",
    }
}

fn pronoun_group_hint_en(zh: &str) -> &'static str {
    match zh {
        "题面是人称，填写主格、宾格、所有格（阴阳/中性/复数）、反身、反身物主。" => {
            "Prompt is person. Fill nominative, accusative, possessive (m/f, neuter, plural), reflexive, reflexive possessive."
        }
        "同样填写 7 类形式；每行对应一个人称。" => {
            "Fill the same 7 forms; each row corresponds to one person."
        }
        "指示代词按行填写对应原型。" => {
            "Fill the base forms of demonstrative pronouns by row."
        }
        _ => "Fill the required pronoun forms for each row.",
    }
}

fn pronoun_prompt_en(zh: &str) -> &'static str {
    match zh {
        "我" => "I",
        "你" => "You (sg)",
        "他" => "He",
        "她" => "She",
        "我们" => "We",
        "你们" => "You (pl)",
        "他们" => "They (m)",
        "她们" => "They (f)",
        "这" => "This",
        "那" => "That",
        "这些" => "These",
        "那些" => "Those",
        _ => "Pronoun",
    }
}

fn pronoun_field_label_en(zh: &str) -> &'static str {
    match zh {
        "主格" => "Nominative",
        "宾格" => "Accusative",
        "所有格阴阳性" => "Possessive (m/f)",
        "所有格中性" => "Possessive (neuter)",
        "所有格复数" => "Possessive (plural)",
        "反身代词" => "Reflexive",
        "反身物主代词" => "Reflexive Possessive",
        _ => "Form",
    }
}

fn build_pronoun_rows(
    entries: &[WordBankEntry],
    row_configs: &[PronounRowConfig],
) -> (Vec<ResolvedPronounRow>, Vec<String>) {
    // 将“题面配置”解析为真实词条映射，缺项会返回用于 UI 提示。
    let mut rows = Vec::new();
    let mut missing_fields = Vec::new();

    for row_cfg in row_configs {
        let mut resolved_fields = Vec::new();
        for field_cfg in row_cfg.fields {
            let Some(entry) = find_pronoun_entry_by_chinese(entries, field_cfg.chinese) else {
                missing_fields.push(format!("{}-{}", row_cfg.prompt, field_cfg.label));
                continue;
            };
            resolved_fields.push(ResolvedPronounField {
                label: field_cfg.label.to_string(),
                entry_id: entry.id.clone(),
                expected: entry.base_form.clone(),
            });
        }
        if !resolved_fields.is_empty() {
            rows.push(ResolvedPronounRow {
                prompt: row_cfg.prompt.to_string(),
                fields: resolved_fields,
            });
        }
    }

    (rows, missing_fields)
}

fn find_pronoun_entry_by_chinese<'a>(
    entries: &'a [WordBankEntry],
    chinese: &str,
) -> Option<&'a WordBankEntry> {
    entries.iter().find(|entry| {
        entry
            .chinese
            .iter()
            .any(|candidate| normalize_for_compare(candidate) == normalize_for_compare(chinese))
    })
}

fn selected_series_entries(
    lexicon_state: LexiconState,
    practice_state: PracticeState,
) -> Vec<WordBankEntry> {
    // 优先使用 `selected_word_entry_ids` 作为系列题库范围；为空则回退全量。
    let selected_ids = practice_state.selected_word_entry_ids.get_untracked();
    let entries = lexicon_state.entries.get_untracked();
    if selected_ids.is_empty() {
        return entries;
    }
    entries
        .into_iter()
        .filter(|entry| selected_ids.iter().any(|id| id == &entry.id))
        .collect()
}
