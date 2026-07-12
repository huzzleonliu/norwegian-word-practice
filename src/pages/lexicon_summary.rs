use leptos::prelude::*;

use crate::app_state::WordBankState;
use crate::components::mini_console::MiniConsole;
use crate::pages::AppPage;
use crate::structures::pracresult::{AnswerStats, PracticeResult, PracticedWordEntryResult};
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::pracresult_crypto::serialize_practice_result_for_export;

#[component]
pub fn LexiconSummaryPage() -> impl IntoView {
    let set_current_page = expect_context::<WriteSignal<AppPage>>();
    let word_bank_state = expect_context::<WordBankState>();
    let set_temp_practice_result = word_bank_state.set_temp_practice_result;
    let (flow_status, set_flow_status) = signal(String::new());
    let summary_return_page = word_bank_state.summary_return_page;

    let continue_practice_click = move |_| {
        let target_page = summary_return_page.get_untracked();
        set_temp_practice_result.set(PracticeResult::default());
        set_current_page.set(target_page);
    };
    let return_back_click = move |_| {
        set_current_page.set(summary_return_page.get_untracked());
    };
    let export_result_click = move |_| {
        let practice_result = word_bank_state.practice_result.get_untracked();
        let content = match serialize_practice_result_for_export(&practice_result) {
            Ok(content) => content,
            Err(err) => {
                set_flow_status.set(format!("导出失败：{err}"));
                return;
            }
        };
        let filename = today_pracresult_filename();
        match export_pracresult_download(&filename, &content) {
            Ok(()) => set_flow_status.set(format!("导出成功：{filename}")),
            Err(err) => set_flow_status.set(format!("导出失败：{err}")),
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-6 shadow-xl">
                <button
                    type="button"
                    on:click=return_back_click
                    class="absolute right-6 top-6 rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-100 hover:bg-slate-700"
                >
                    "返回上一级页面"
                </button>
                <h1 class="text-2xl font-bold tracking-tight">"练习总结"</h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let session_result = word_bank_state.last_completed_practice_result.get();
                        let (session_correct, session_wrong) = aggregate_score(&session_result);
                        let session_total = session_correct + session_wrong;
                        let summary_line = format!(
                            "本次结果：准确率 {}（{session_correct}/{session_total}）",
                            format_accuracy(session_correct, session_total)
                        );
                        let flow_line = {
                            let s = flow_status.get();
                            if s.trim().is_empty() {
                                "等待导出或继续下一步...".to_string()
                            } else {
                                s
                            }
                        };
                        format!("{summary_line}\n{flow_line}")
                    })
                />

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"练习总结部分"</h2>

                    <div class="mt-4 rounded-lg border border-slate-800 bg-slate-900/60 p-4">
                        <h3 class="text-base font-semibold text-slate-200">"本次练习结果"</h3>
                        <p class="mt-2 text-sm text-slate-300">
                            {move || {
                                let session_result = word_bank_state.last_completed_practice_result.get();
                                let (correct, wrong) = aggregate_score(&session_result);
                                let total = correct + wrong;
                                format!(
                                    "总体准确率：{}（{correct}/{total}）",
                                    format_accuracy(correct, total)
                                )
                            }}
                        </p>

                        <div class="mt-3">
                            <p class="text-sm text-slate-300">"错过词条（中文 / 词性 / 正确答案 / 错误回答）"</p>
                            {move || {
                                let session_result = word_bank_state.last_completed_practice_result.get();
                                let entries = word_bank_state.entries.get();
                                let wrong_items = build_wrong_summary_items(&session_result, &entries);
                                if wrong_items.is_empty() {
                                    view! {
                                        <p class="mt-2 text-sm text-emerald-300">"本次没有错误词条。"</p>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <ul class="mt-2 space-y-2 text-sm text-slate-300">
                                            {wrong_items
                                                .into_iter()
                                                .map(|item| {
                                                    view! {
                                                        <li class="rounded border border-slate-800 bg-slate-950/70 px-3 py-2">
                                                            <p>{format!("中文：{}", item.chinese)}</p>
                                                            <p>{format!("词性：{}", item.part_of_speech)}</p>
                                                            <p>{format!("正确答案：{}", item.correct_answer)}</p>
                                                            <p>{format!("错误回答：{}", item.wrong_answers)}</p>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    }
                                        .into_any()
                                }
                            }}
                        </div>
                    </div>

                    <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
                        <div class="rounded-lg border border-slate-800 bg-slate-900/60 p-4">
                            <h3 class="text-base font-semibold text-slate-200">"历史数据（不含本次）"</h3>
                            <p class="mt-2 text-sm text-slate-300">
                                {move || {
                                    let total_result = word_bank_state.practice_result.get();
                                    let session_result = word_bank_state.last_completed_practice_result.get();
                                    let (total_correct, total_wrong) = aggregate_score(&total_result);
                                    let (session_correct, session_wrong) = aggregate_score(&session_result);
                                    let history_correct = total_correct.saturating_sub(session_correct);
                                    let history_wrong = total_wrong.saturating_sub(session_wrong);
                                    let history_total = history_correct + history_wrong;

                                    format!(
                                        "准确率：{}（正确：{history_correct}，错误：{history_wrong}，总数：{history_total}）",
                                        format_accuracy(history_correct, history_total)
                                    )
                                }}
                            </p>
                        </div>

                        <div class="rounded-lg border border-slate-800 bg-slate-900/60 p-4">
                            <h3 class="text-base font-semibold text-slate-200">"总数据（含本次）"</h3>
                            <p class="mt-2 text-sm text-slate-300">
                                {move || {
                                    let total_result = word_bank_state.practice_result.get();
                                    let (total_correct, total_wrong) = aggregate_score(&total_result);
                                    let total = total_correct + total_wrong;
                                    format!(
                                        "准确率：{}（正确：{total_correct}，错误：{total_wrong}，总数：{total}）",
                                        format_accuracy(total_correct, total)
                                    )
                                }}
                            </p>
                        </div>
                    </div>
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">"流程控制部分"</h2>
                    <div class="mt-4 flex flex-wrap items-center gap-3">
                        <button
                            type="button"
                            on:click=export_result_click
                            class="rounded-lg border border-cyan-700 bg-cyan-700 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-600"
                        >
                            "导出练习结果"
                        </button>
                        <button
                            type="button"
                            on:click=continue_practice_click
                            class="rounded-lg border border-emerald-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600"
                        >
                            "继续练习"
                        </button>
                    </div>
                </section>
            </section>
        </main>
    }
}

#[derive(Clone)]
struct WrongSummaryItem {
    chinese: String,
    part_of_speech: String,
    correct_answer: String,
    wrong_answers: String,
}

fn build_wrong_summary_items(
    session_result: &PracticeResult,
    entries: &[WordBankEntry],
) -> Vec<WrongSummaryItem> {
    session_result
        .practiced_word_entries
        .iter()
        .filter(|entry| practiced_entry_has_wrong(entry))
        .map(|practiced_entry| {
            if let Some(word_entry) = entries.iter().find(|entry| entry.id == practiced_entry.id) {
                WrongSummaryItem {
                    chinese: join_or_placeholder(&word_entry.chinese),
                    part_of_speech: word_entry.part_of_speech.as_key().to_string(),
                    correct_answer: build_part_of_speech_correct_answer(word_entry),
                    wrong_answers: build_wrong_answers_summary(practiced_entry),
                }
            } else {
                WrongSummaryItem {
                    chinese: "（词库中未找到）".to_string(),
                    part_of_speech: "（未知）".to_string(),
                    correct_answer: "（无法给出正确答案）".to_string(),
                    wrong_answers: build_wrong_answers_summary(practiced_entry),
                }
            }
        })
        .collect()
}

fn build_wrong_answers_summary(practiced_entry: &PracticedWordEntryResult) -> String {
    let mut parts = Vec::<String>::new();
    for (label, stats) in practiced_entry_stats_with_labels(practiced_entry) {
        if stats.wrong_count == 0 {
            continue;
        }

        let wrong_text = if stats.wrong_answers.is_empty() {
            "（未记录具体输入）".to_string()
        } else {
            stats.wrong_answers.join(" / ")
        };
        parts.push(format!("{label}: {wrong_text}"));
    }

    if parts.is_empty() {
        "（无）".to_string()
    } else {
        parts.join("；")
    }
}

fn join_or_placeholder(values: &[String]) -> String {
    let joined = values
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    if joined.is_empty() {
        "（无）".to_string()
    } else {
        joined
    }
}

fn build_part_of_speech_correct_answer(entry: &WordBankEntry) -> String {
    let mut parts = Vec::<String>::new();
    push_answer_part(&mut parts, "原型", Some(entry.base_form.as_str()));

    match &entry.part_of_speech {
        PartOfSpeech::Verb => {
            push_answer_part(&mut parts, "现在时", entry.verb_present_tense.as_deref());
            push_answer_part(&mut parts, "过去式", entry.verb_past_tense.as_deref());
            push_answer_part(&mut parts, "祈使式", entry.verb_imperative.as_deref());
        }
        PartOfSpeech::Noun => {
            push_answer_part(&mut parts, "复数", entry.noun_plural.as_deref());
            push_answer_part(&mut parts, "单数特指", entry.noun_singular_definite.as_deref());
            push_answer_part(&mut parts, "复数特指", entry.noun_plural_definite.as_deref());
        }
        PartOfSpeech::Adjective => {
            push_answer_part(&mut parts, "对应中性", entry.adjective_neuter_form.as_deref());
            push_answer_part(&mut parts, "对应复数", entry.adjective_plural_form.as_deref());
            push_answer_part(&mut parts, "比较级", entry.adjective_comparative.as_deref());
            push_answer_part(
                &mut parts,
                "最高级泛指",
                entry.adjective_superlative_indefinite.as_deref(),
            );
            push_answer_part(
                &mut parts,
                "最高级特指",
                entry.adjective_superlative_definite.as_deref(),
            );
        }
        PartOfSpeech::Adverb => {
            push_answer_part(&mut parts, "比较级", entry.adverb_comparative.as_deref());
            push_answer_part(&mut parts, "最高级", entry.adverb_superlative.as_deref());
        }
        PartOfSpeech::CardinalNumber
        | PartOfSpeech::OrdinalNumber
        | PartOfSpeech::Month
        | PartOfSpeech::Pronoun
        | PartOfSpeech::Interrogative => {}
    }

    if parts.is_empty() {
        "（无可展示答案）".to_string()
    } else {
        parts.join("；")
    }
}

fn push_answer_part(parts: &mut Vec<String>, label: &str, value: Option<&str>) {
    let value = value.unwrap_or("").trim();
    if !value.is_empty() {
        parts.push(format!("{label}: {value}"));
    }
}

fn aggregate_score(result: &PracticeResult) -> (usize, usize) {
    let mut correct = 0_usize;
    let mut wrong = 0_usize;

    for practiced_entry in &result.practiced_word_entries {
        for stats in practiced_entry_stats(practiced_entry) {
            correct += stats.correct_count;
            wrong += stats.wrong_count;
        }
    }

    (correct, wrong)
}

fn practiced_entry_has_wrong(practiced_entry: &PracticedWordEntryResult) -> bool {
    practiced_entry_stats(practiced_entry)
        .iter()
        .any(|stats| stats.wrong_count > 0)
}

fn practiced_entry_stats(practiced_entry: &PracticedWordEntryResult) -> [&AnswerStats; 16] {
    practiced_entry_stats_with_labels(practiced_entry).map(|(_, stats)| stats)
}

fn practiced_entry_stats_with_labels(
    practiced_entry: &PracticedWordEntryResult,
) -> [(&'static str, &AnswerStats); 16] {
    [
        ("english", &practiced_entry.english),
        ("chinese", &practiced_entry.chinese),
        ("base_form", &practiced_entry.base_form),
        ("verb_present_tense", &practiced_entry.verb_present_tense),
        ("verb_past_tense", &practiced_entry.verb_past_tense),
        ("verb_imperative", &practiced_entry.verb_imperative),
        ("noun_plural", &practiced_entry.noun_plural),
        ("noun_singular_definite", &practiced_entry.noun_singular_definite),
        ("noun_plural_definite", &practiced_entry.noun_plural_definite),
        ("adjective_neuter_form", &practiced_entry.adjective_neuter_form),
        ("adjective_plural_form", &practiced_entry.adjective_plural_form),
        (
            "adjective_comparative",
            &practiced_entry.adjective_comparative,
        ),
        (
            "adjective_superlative_indefinite",
            &practiced_entry.adjective_superlative_indefinite,
        ),
        (
            "adjective_superlative_definite",
            &practiced_entry.adjective_superlative_definite,
        ),
        ("adverb_comparative", &practiced_entry.adverb_comparative),
        ("adverb_superlative", &practiced_entry.adverb_superlative),
    ]
}

fn format_accuracy(correct: usize, total: usize) -> String {
    if total == 0 {
        return "0.00%".to_string();
    }
    let rate = (correct as f64 / total as f64) * 100.0;
    format!("{rate:.2}%")
}

fn today_pracresult_filename() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let now = js_sys::Date::new_0();
        let year = now.get_full_year() as i32;
        let month = now.get_month() + 1;
        let day = now.get_date();
        format!("{year:04}-{month:02}-{day:02}.pracresult")
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "practice-result.pracresult".to_string()
    }
}

fn export_pracresult_download(filename: &str, content: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue};

        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(content));
        let blob = web_sys::Blob::new_with_str_sequence(&parts)
            .map_err(|_| "无法创建练习结果 Blob".to_string())?;
        let object_url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "无法创建练习结果下载 URL".to_string())?;

        let window = web_sys::window().ok_or_else(|| "无法获取 window".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "无法获取 document".to_string())?;
        let anchor = document
            .create_element("a")
            .map_err(|_| "无法创建下载节点".to_string())?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "无法转换下载节点".to_string())?;

        anchor.set_href(&object_url);
        anchor.set_download(filename);

        let body = document
            .body()
            .ok_or_else(|| "页面 body 不存在".to_string())?;
        body.append_child(&anchor)
            .map_err(|_| "无法挂载下载节点".to_string())?;
        anchor.click();
        anchor.remove();

        web_sys::Url::revoke_object_url(&object_url)
            .map_err(|_| "无法释放练习结果下载 URL".to_string())?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filename;
        let _ = content;
        Err("导出仅在浏览器环境可用。".to_string())
    }
}
