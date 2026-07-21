//! 练习总结页：展示本次结果、历史聚合、错题信息，并支持导出 `.pracresult`。

use leptos::prelude::*;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::structures::field_meta::for_each_answer_stats;
use crate::structures::pracresult::{PracticeResult, PracticedWordEntryResult};
use crate::structures::word_bank_entry::{PartOfSpeech, UiLanguage, WordBankEntry};
use crate::utils::i18n::{field_label, tr};
use crate::utils::pracresult_crypto::serialize_practice_result_for_export;

#[component]
pub fn LexiconSummaryPage() -> impl IntoView {
    let set_current_page = expect_context::<NavigateToPage>();
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;
    let set_temp_practice_result = practice_state.set_temp_practice_result;
    let (flow_status, set_flow_status) = signal(String::new());
    let summary_return_page = practice_state.summary_return_page;

    let continue_practice_click = move |_| {
        let target_page = summary_return_page.get_untracked();
        set_temp_practice_result.set(PracticeResult::default());
        set_current_page.set(target_page);
    };
    let return_back_click = move |_| {
        set_current_page.set(summary_return_page.get_untracked());
    };
    let export_result_click = move |_| {
        let language = lang.get_untracked();
        let practice_result = practice_state.practice_result.get_untracked();
        let content = match serialize_practice_result_for_export(&practice_result) {
            Ok(content) => content,
            Err(err) => {
                set_flow_status.set(format!(
                    "{} {err}",
                    tr(language, "导出失败：", "Export failed:")
                ));
                return;
            }
        };
        let filename = today_pracresult_filename();
        match export_pracresult_download(&filename, &content) {
            Ok(()) => set_flow_status.set(format!(
                "{}：{filename}",
                tr(language, "导出成功", "Export success")
            )),
            Err(err) => set_flow_status.set(format!(
                "{} {err}",
                tr(language, "导出失败：", "Export failed:")
            )),
        }
    };

    view! {
        <main class="min-h-screen bg-slate-950 text-slate-100 p-3 sm:p-6">
            <section class="relative mx-auto w-full max-w-6xl rounded-2xl border border-slate-800 bg-slate-900 p-4 sm:p-6 shadow-xl">
                <button
                    type="button"
                    on:click=return_back_click
                    class="mb-4 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-100 hover:bg-slate-700 sm:absolute sm:right-6 sm:top-6 sm:mb-0 sm:w-auto"
                >
                    {move || tr(lang.get(), "返回上一级页面", "Back")}
                </button>
                <h1 class="text-xl sm:text-2xl font-bold tracking-tight">{move || tr(lang.get(), "练习总结", "Practice Summary")}</h1>
                <MiniConsole
                    message=Signal::derive(move || {
                        let language = lang.get();
                        let session_result = practice_state.last_completed_practice_result.get();
                        let (session_correct, session_wrong) = aggregate_score(&session_result);
                        let session_total = session_correct + session_wrong;
                        let summary_line = format!(
                            "{}: {} {} ({session_correct}/{session_total})",
                            tr(language, "本次结果", "Current Session"),
                            tr(language, "准确率", "Accuracy"),
                            format_accuracy(session_correct, session_total)
                        );
                        let flow_line = {
                            let s = flow_status.get();
                            if s.trim().is_empty() {
                                tr(language, "等待导出或继续下一步...", "Waiting for export or next step...")
                                    .to_string()
                            } else {
                                s
                            }
                        };
                        format!("{summary_line}\n{flow_line}")
                    })
                />

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">{move || tr(lang.get(), "练习总结部分", "Summary Section")}</h2>

                    <div class="mt-4 rounded-lg border border-slate-800 bg-slate-900/60 p-4">
                        <h3 class="text-base font-semibold text-slate-200">{move || tr(lang.get(), "本次练习结果", "Current Session Result")}</h3>
                        <p class="mt-2 text-sm text-slate-300">
                            {move || {
                                let language = lang.get();
                                let session_result = practice_state.last_completed_practice_result.get();
                                let (correct, wrong) = aggregate_score(&session_result);
                                let total = correct + wrong;
                                format!(
                                    "{}: {} ({correct}/{total})",
                                    tr(language, "总体准确率", "Overall accuracy"),
                                    format_accuracy(correct, total)
                                )
                            }}
                        </p>

                        <div class="mt-3">
                            <p class="text-sm text-slate-300">
                                {move || {
                                    tr(
                                        lang.get(),
                                        "错过词条（中文 / 词性 / 正确答案 / 错误回答）",
                                        "Wrong Entries (Chinese / POS / Correct / Wrong Inputs)",
                                    )
                                }}
                            </p>
                            {move || {
                                let language = lang.get();
                                let session_result = practice_state.last_completed_practice_result.get();
                                let entries = lexicon_state.entries.get();
                                let wrong_items =
                                    build_wrong_summary_items(&session_result, &entries, language);
                                if wrong_items.is_empty() {
                                    view! {
                                        <p class="mt-2 text-sm text-emerald-300">
                                            {move || tr(lang.get(), "本次没有错误词条。", "No wrong entries in this session.")}
                                        </p>
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
                                                            <p>{move || format!("{}: {}", tr(lang.get(), "中文", "Chinese"), item.chinese)}</p>
                                                            <p>{move || format!("{}: {}", tr(lang.get(), "词性", "Part of Speech"), item.part_of_speech)}</p>
                                                            <p>{move || format!("{}: {}", tr(lang.get(), "正确答案", "Correct"), item.correct_answer)}</p>
                                                            <p>{move || format!("{}: {}", tr(lang.get(), "错误回答", "Wrong Inputs"), item.wrong_answers)}</p>
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
                            <h3 class="text-base font-semibold text-slate-200">{move || tr(lang.get(), "历史数据（不含本次）", "History (excluding current)")}</h3>
                            <p class="mt-2 text-sm text-slate-300">
                                {move || {
                                    let language = lang.get();
                                    let total_result = practice_state.practice_result.get();
                                    let session_result = practice_state.last_completed_practice_result.get();
                                    let (total_correct, total_wrong) = aggregate_score(&total_result);
                                    let (session_correct, session_wrong) = aggregate_score(&session_result);
                                    let history_correct = total_correct.saturating_sub(session_correct);
                                    let history_wrong = total_wrong.saturating_sub(session_wrong);
                                    let history_total = history_correct + history_wrong;

                                    format!(
                                        "{}: {} ({}: {history_correct}, {}: {history_wrong}, {}: {history_total})",
                                        tr(language, "准确率", "Accuracy"),
                                        format_accuracy(history_correct, history_total)
                                        ,
                                        tr(language, "正确", "Correct"),
                                        tr(language, "错误", "Wrong"),
                                        tr(language, "总数", "Total")
                                    )
                                }}
                            </p>
                        </div>

                        <div class="rounded-lg border border-slate-800 bg-slate-900/60 p-4">
                            <h3 class="text-base font-semibold text-slate-200">{move || tr(lang.get(), "总数据（含本次）", "All Data (including current)")}</h3>
                            <p class="mt-2 text-sm text-slate-300">
                                {move || {
                                    let language = lang.get();
                                    let total_result = practice_state.practice_result.get();
                                    let (total_correct, total_wrong) = aggregate_score(&total_result);
                                    let total = total_correct + total_wrong;
                                    format!(
                                        "{}: {} ({}: {total_correct}, {}: {total_wrong}, {}: {total})",
                                        tr(language, "准确率", "Accuracy"),
                                        format_accuracy(total_correct, total)
                                        ,
                                        tr(language, "正确", "Correct"),
                                        tr(language, "错误", "Wrong"),
                                        tr(language, "总数", "Total")
                                    )
                                }}
                            </p>
                        </div>
                    </div>
                </section>

                <section class="mt-6 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                    <h2 class="text-lg font-semibold">{move || tr(lang.get(), "流程控制部分", "Flow Control")}</h2>
                    <div class="mt-4 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                        <button
                            type="button"
                            on:click=export_result_click
                            class="w-full rounded-lg border border-cyan-700 bg-cyan-700 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-600 sm:w-auto"
                        >
                            {move || tr(lang.get(), "导出练习结果", "Export Practice Result")}
                        </button>
                        <button
                            type="button"
                            on:click=continue_practice_click
                            class="w-full rounded-lg border border-emerald-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 sm:w-auto"
                        >
                            {move || tr(lang.get(), "继续练习", "Continue Practice")}
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
    language: UiLanguage,
) -> Vec<WrongSummaryItem> {
    // 仅提取“本轮存在错误”的词条，拼装为可展示的错题摘要。
    session_result
        .practiced_word_entries
        .iter()
        .filter(|entry| practiced_entry_has_wrong(entry))
        .map(|practiced_entry| {
            if let Some(word_entry) = entries.iter().find(|entry| entry.id == practiced_entry.id) {
                WrongSummaryItem {
                    chinese: join_or_placeholder(&word_entry.chinese, language),
                    part_of_speech: word_entry.part_of_speech.display_name(language).to_string(),
                    correct_answer: build_part_of_speech_correct_answer(word_entry, language),
                    wrong_answers: build_wrong_answers_summary(practiced_entry, language),
                }
            } else {
                WrongSummaryItem {
                    chinese: tr(language, "（词库中未找到）", "(Not found in lexicon)").to_string(),
                    part_of_speech: tr(language, "（未知）", "(Unknown)").to_string(),
                    correct_answer: tr(
                        language,
                        "（无法给出正确答案）",
                        "(No valid correct answer)",
                    )
                    .to_string(),
                    wrong_answers: build_wrong_answers_summary(practiced_entry, language),
                }
            }
        })
        .collect()
}

fn build_wrong_answers_summary(
    practiced_entry: &PracticedWordEntryResult,
    language: UiLanguage,
) -> String {
    let mut parts = Vec::<String>::new();
    for_each_answer_stats(practiced_entry, |label, stats| {
        if stats.wrong_count == 0 {
            return;
        }

        let wrong_text = if stats.wrong_answers.is_empty() {
            tr(language, "（未记录具体输入）", "(No recorded input)").to_string()
        } else {
            stats.wrong_answers.join(" / ")
        };
        parts.push(format!("{}: {wrong_text}", field_label(language, label)));
    });

    if parts.is_empty() {
        tr(language, "（无）", "(None)").to_string()
    } else {
        parts.join("; ")
    }
}

fn join_or_placeholder(values: &[String], language: UiLanguage) -> String {
    let joined = values
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    if joined.is_empty() {
        tr(language, "（无）", "(None)").to_string()
    } else {
        joined
    }
}

fn build_part_of_speech_correct_answer(entry: &WordBankEntry, language: UiLanguage) -> String {
    // 按词性拼装“正确答案”文本，避免展示无关空字段。
    let mut parts = Vec::<String>::new();
    push_answer_part(
        &mut parts,
        tr(language, "原型", "Base Form"),
        Some(entry.base_form.as_str()),
    );

    match &entry.part_of_speech {
        PartOfSpeech::Verb => {
            push_answer_part(
                &mut parts,
                tr(language, "现在时", "Present"),
                entry.verb_present_tense.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "过去式", "Past"),
                entry.verb_past_tense.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "祈使式", "Imperative"),
                entry.verb_imperative.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "现在分词", "Present Participle"),
                entry.verb_present_participle.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "过去分词", "Past Participle"),
                entry.verb_past_participle.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "被动不定式", "Passive Infinitive"),
                entry.verb_passive_infinitive.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "被动现在时", "Passive Present"),
                entry.verb_passive_present.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "被动过去时", "Passive Past"),
                entry.verb_passive_past.as_deref(),
            );
        }
        PartOfSpeech::Noun => {
            push_answer_part(
                &mut parts,
                tr(language, "复数", "Plural"),
                entry.noun_plural.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "单数特指", "Singular Definite"),
                entry.noun_singular_definite.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数特指", "Plural Definite"),
                entry.noun_plural_definite.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "单数特指所有格", "Singular Definite Genitive"),
                entry.noun_singular_definite_genitive.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数特指所有格", "Plural Definite Genitive"),
                entry.noun_plural_definite_genitive.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "单数泛指所有格", "Singular Indefinite Genitive"),
                entry.noun_singular_indefinite_genitive.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数泛指所有格", "Plural Indefinite Genitive"),
                entry.noun_plural_indefinite_genitive.as_deref(),
            );
        }
        PartOfSpeech::Adjective => {
            push_answer_part(
                &mut parts,
                tr(language, "对应阴性", "Feminine"),
                entry.adjective_feminine_form.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "对应中性", "Neuter"),
                entry.adjective_neuter_form.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "对应复数", "Plural"),
                entry.adjective_plural_form.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "比较级", "Comparative"),
                entry.adjective_comparative.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "最高级泛指", "Superlative Indefinite"),
                entry.adjective_superlative_indefinite.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "最高级特指", "Superlative Definite"),
                entry.adjective_superlative_definite.as_deref(),
            );
        }
        PartOfSpeech::Pronoun => {
            push_answer_part(
                &mut parts,
                tr(language, "宾格", "Object"),
                entry.pronoun_object.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "反身", "Reflexive"),
                entry.pronoun_reflexive.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数主格", "Plural Subject"),
                entry.pronoun_plural_subject.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数宾格", "Plural Object"),
                entry.pronoun_plural_object.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数反身", "Plural Reflexive"),
                entry.pronoun_plural_reflexive.as_deref(),
            );
        }
        PartOfSpeech::Determinative => {
            push_answer_part(
                &mut parts,
                tr(language, "阴性", "Feminine"),
                entry.determinative_feminine_form.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "中性", "Neuter"),
                entry.determinative_neuter_form.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "复数", "Plural"),
                entry.determinative_plural_form.as_deref(),
            );
        }
        PartOfSpeech::Adverb => {
            push_answer_part(
                &mut parts,
                tr(language, "比较级", "Comparative"),
                entry.adverb_comparative.as_deref(),
            );
            push_answer_part(
                &mut parts,
                tr(language, "最高级", "Superlative"),
                entry.adverb_superlative.as_deref(),
            );
        }
        PartOfSpeech::Preposition
        | PartOfSpeech::Conjunction
        | PartOfSpeech::Subjunction
        | PartOfSpeech::Interjection
        | PartOfSpeech::Phrase => {}
    }

    if parts.is_empty() {
        tr(language, "（无可展示答案）", "(No displayable answer)").to_string()
    } else {
        parts.join("; ")
    }
}

fn push_answer_part(parts: &mut Vec<String>, label: &str, value: Option<&str>) {
    let value = value.unwrap_or("").trim();
    if !value.is_empty() {
        parts.push(format!("{label}: {value}"));
    }
}

fn aggregate_score(result: &PracticeResult) -> (usize, usize) {
    // 聚合所有字段统计：返回 (correct, wrong)。
    let mut correct = 0_usize;
    let mut wrong = 0_usize;

    for practiced_entry in &result.practiced_word_entries {
        for_each_answer_stats(practiced_entry, |_, stats| {
            correct += stats.correct_count;
            wrong += stats.wrong_count;
        });
    }

    (correct, wrong)
}

fn practiced_entry_has_wrong(practiced_entry: &PracticedWordEntryResult) -> bool {
    let mut has_wrong = false;
    for_each_answer_stats(practiced_entry, |_, stats| {
        if stats.wrong_count > 0 {
            has_wrong = true;
        }
    });
    has_wrong
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
            .map_err(|_| "Failed to create practice result Blob".to_string())?;
        let object_url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create practice result download URL".to_string())?;

        let window = web_sys::window().ok_or_else(|| "Failed to get window".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "Failed to get document".to_string())?;
        let anchor = document
            .create_element("a")
            .map_err(|_| "Failed to create download node".to_string())?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| "Failed to cast download node".to_string())?;

        anchor.set_href(&object_url);
        anchor.set_download(filename);

        let body = document
            .body()
            .ok_or_else(|| "Missing page body".to_string())?;
        body.append_child(&anchor)
            .map_err(|_| "Failed to mount download node".to_string())?;
        anchor.click();
        anchor.remove();

        web_sys::Url::revoke_object_url(&object_url)
            .map_err(|_| "Failed to release practice result download URL".to_string())?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filename;
        let _ = content;
        Err("Export is only available in browser environment.".to_string())
    }
}
