//! 国家系列练习页：按 tag 将同一国家相关词条归入同一块，块标题为国家名。

use std::collections::{HashMap, HashSet};

use leptos::prelude::*;

use crate::app_state::{LexiconState, NavigateToPage, PracticeState, UiState};
use crate::components::mini_console::MiniConsole;
use crate::components::practice_engine::{
    AbortPracticeButton, AnswerFieldsSettings, CheckAnswersError, CheckPracticeButton,
    FinishPracticeButton, PracticeEntry, RestartPracticeButton, RestartTempBehavior,
    apply_check_results, default_answer_fields, evaluate_check_answers, merge_solved_question_ids,
    prepare_post_check_input_state, retain_unsolved_revealed_keys,
};
use crate::layout::{PageHeading, PageShell, PageTitle, PageTopbar, ShellAttach};
use crate::pages::AppPage;
use crate::structures::word_bank_entry::{PartOfSpeech, UiLanguage, WordBankEntry};
use crate::utils::i18n::tr;

use super::initialize_temp_practice_result;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CountryWordRole {
    CountryName = 0,
    Language = 1,
    Adjective = 2,
    Person = 3,
    Other = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CountryBlock {
    /// tags 中的挪威语国家名（如 `Norge`）。
    country_tag: String,
    title_zh: String,
    title_en: String,
    entry_ids: Vec<String>,
}

#[component]
pub fn CountrySerisePracticePage() -> impl IntoView {
    let lexicon_state = expect_context::<LexiconState>();
    let practice_state = expect_context::<PracticeState>();
    let lang = expect_context::<UiState>().ui_language;
    initialize_temp_practice_result(lexicon_state, practice_state);

    let set_current_page = expect_context::<NavigateToPage>();
    let (prompt_field_a, _) = signal("chinese".to_string());
    let (prompt_field_b, _) = signal("english".to_string());
    let (prompt_field_c, _) = signal("part_of_speech".to_string());
    let (answer_fields, set_answer_fields) = signal(default_answer_fields());
    let (allow_answer_reveal, set_allow_answer_reveal) = signal(true);
    let (revealed_answer_keys, set_revealed_answer_keys) = signal(HashSet::<String>::new());
    let (status, set_status) = signal(String::new());
    let (answer_inputs, set_answer_inputs) = signal(HashMap::<String, String>::new());
    let (wrong_answer_feedback, set_wrong_answer_feedback) =
        signal(HashMap::<String, String>::new());
    let (solved_question_ids, set_solved_question_ids) = signal(Vec::<String>::new());

    let country_blocks = Memo::new(move |_| {
        let entries = lexicon_state.entries.get();
        let selected_ids = practice_state.selected_word_entry_ids.get();
        let solved = solved_question_ids.get();
        let solved_set: HashSet<&str> = solved.iter().map(|id| id.as_str()).collect();
        build_country_blocks(&entries, &selected_ids)
            .into_iter()
            .filter_map(|mut block| {
                block
                    .entry_ids
                    .retain(|id| !solved_set.contains(id.as_str()));
                if block.entry_ids.is_empty() {
                    None
                } else {
                    Some(block)
                }
            })
            .collect::<Vec<_>>()
    });

    let all_active_ids = Memo::new(move |_| {
        country_blocks
            .get()
            .into_iter()
            .flat_map(|block| block.entry_ids)
            .collect::<Vec<_>>()
    });

    let check_click = Callback::new(move |_| {
        let language = lang.get_untracked();
        let selected_answer_fields = answer_fields.get_untracked();
        let active_ids = all_active_ids.get_untracked();
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
                tr(language, "条。", "entries.")
            ));
        }
    });

    let restart_ui_click = Callback::new(move |_| {
        set_solved_question_ids.set(Vec::new());
        set_answer_inputs.set(HashMap::new());
        set_revealed_answer_keys.set(HashSet::new());
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
                        {move || tr(lang.get(), "国家系列练习", "Country Series Practice")}
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
                        let blocks = country_blocks.get().len();
                        let overview = format!(
                            "{}：{selected_count} {}，{} {blocks} {}，{} {temp_entries} {}。",
                            tr(language, "当前可练习词条", "Available entries"),
                            tr(language, "条", "entries"),
                            tr(language, "国家块", "country blocks"),
                            tr(language, "个", ""),
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
                        {move || tr(lang.get(), "第一部分：练习设置", "Part 1: Practice Settings")}
                    </h2>
                    <div class="mt-4">
                        <AnswerFieldsSettings
                            answer_fields=answer_fields
                            set_answer_fields=set_answer_fields
                        />
                    </div>
                    <div class="mt-4 rounded border border-slate-800 bg-slate-950/40 p-3">
                        <p class="text-sm text-slate-300">
                            {move || tr(lang.get(), "答案显隐", "Answer Reveal")}
                        </p>
                        <div class="mt-2 flex flex-wrap items-center gap-3">
                            <label class="inline-flex items-center gap-2 text-xs text-slate-300">
                                <input
                                    type="checkbox"
                                    prop:checked=move || allow_answer_reveal.get()
                                    on:change=move |ev| {
                                        set_allow_answer_reveal.set(event_target_checked(&ev))
                                    }
                                />
                                <span>
                                    {move || {
                                        tr(
                                            lang.get(),
                                            "允许在练习区临时亮出答案（默认隐藏）",
                                            "Allow temporary answer reveal (hidden by default)",
                                        )
                                    }}
                                </span>
                            </label>
                            <button
                                type="button"
                                on:click=move |_| hide_all_revealed_answers.run(())
                                class="rounded border border-slate-700 bg-slate-900 px-2 py-1 text-xs text-slate-200 hover:bg-slate-800"
                            >
                                {move || {
                                    tr(lang.get(), "隐藏所有已亮出的答案", "Hide all revealed answers")
                                }}
                            </button>
                        </div>
                    </div>
                </section>

                <section class="mt-6 space-y-4">
                    <h2 class="text-lg font-semibold">
                        {move || tr(lang.get(), "第二部分：练习题", "Part 2: Questions")}
                    </h2>
                    <p class="text-sm text-slate-400">
                        {move || {
                            tr(
                                lang.get(),
                                "每个国家一块：国名、语言、形容词与国人相关词条同组练习。",
                                "Each country is one block: country name, language, adjective, and person forms together.",
                            )
                        }}
                    </p>

                    <For
                        each=move || country_blocks.get()
                        key=|block| {
                            format!("{}::{}", block.country_tag, block.entry_ids.join("|"))
                        }
                        children=move |block| {
                            let title_zh = block.title_zh.clone();
                            let title_en = block.title_en.clone();
                            let (block_ids, _) = signal(block.entry_ids.clone());
                            view! {
                                <section class="rounded-xl border border-slate-800 bg-slate-950/50 p-4">
                                    <h3 class="text-xl font-bold tracking-tight text-slate-100">
                                        {move || match lang.get() {
                                            UiLanguage::Zh => title_zh.clone(),
                                            UiLanguage::En => title_en.clone(),
                                        }}
                                    </h3>
                                    <PracticeEntry
                                        on_check=check_click
                                        active_question_ids=block_ids
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
                                        show_check_button=false
                                    />
                                </section>
                            }
                        }
                    />

                    {move || {
                        if country_blocks.get().is_empty() {
                            view! {
                                <p class="text-sm text-amber-300">
                                    {move || {
                                        tr(
                                            lang.get(),
                                            "当前没有可练习题目，请返回上页先选择国家系列。",
                                            "No questions available. Please go back and open the country series.",
                                        )
                                    }}
                                </p>
                            }
                                .into_any()
                        } else {
                            view! { <></> }.into_any()
                        }
                    }}

                    <div class="flex justify-center pt-2">
                        <CheckPracticeButton on_check=check_click/>
                    </div>
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
                            summary_return_page=AppPage::SeriseCountryPractice
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
        </PageShell>
    }
}

fn build_country_blocks(entries: &[WordBankEntry], selected_ids: &[String]) -> Vec<CountryBlock> {
    let selected: HashSet<&str> = selected_ids.iter().map(|id| id.as_str()).collect();
    let relevant: Vec<&WordBankEntry> = entries
        .iter()
        .filter(|entry| selected.contains(entry.id.as_str()))
        .filter(|entry| entry.tags.iter().any(|tag| tag == "country"))
        .collect();

    let mut country_tags: Vec<String> = relevant
        .iter()
        .flat_map(|entry| country_names_from_tags(&entry.tags))
        .collect();
    country_tags.sort();
    country_tags.dedup();

    country_tags
        .into_iter()
        .filter_map(|country_tag| {
            let mut members: Vec<&WordBankEntry> = relevant
                .iter()
                .copied()
                .filter(|entry| country_names_from_tags(&entry.tags).contains(&country_tag))
                .collect();
            if members.is_empty() {
                return None;
            }
            members.sort_by_key(|entry| {
                (
                    classify_country_word_role(entry, &country_tag) as u8,
                    entry.base_form.clone(),
                    entry.id.clone(),
                )
            });

            let name_entry = members.iter().find(|entry| {
                classify_country_word_role(entry, &country_tag) == CountryWordRole::CountryName
            });
            let title_zh = name_entry
                .and_then(|entry| entry.chinese.first())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or(country_tag.as_str())
                .to_string();
            let title_en = name_entry
                .and_then(|entry| entry.english.first())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or(country_tag.as_str())
                .to_string();

            Some(CountryBlock {
                country_tag,
                title_zh,
                title_en,
                entry_ids: members.into_iter().map(|entry| entry.id.clone()).collect(),
            })
        })
        .collect()
}

fn country_names_from_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .filter(|tag| tag.as_str() != "country")
        .cloned()
        .collect()
}

fn classify_country_word_role(entry: &WordBankEntry, country_tag: &str) -> CountryWordRole {
    if entry.part_of_speech == PartOfSpeech::Adjective {
        return CountryWordRole::Adjective;
    }
    if entry.part_of_speech != PartOfSpeech::Noun {
        return CountryWordRole::Other;
    }
    if entry.base_form.eq_ignore_ascii_case(country_tag) {
        return CountryWordRole::CountryName;
    }

    let zh_joined = entry.chinese.join("");
    let base = entry.base_form.trim();
    let base_lower = base.to_lowercase();
    if zh_joined.contains('语') || looks_like_language_base(&base_lower) {
        return CountryWordRole::Language;
    }
    if entry.chinese.iter().any(|zh| zh.trim().ends_with('人'))
        || looks_like_person_base(&base_lower)
    {
        return CountryWordRole::Person;
    }
    // 无明确线索时：剩余名词默认按国人处理（语言通常已由 -sk / 「语」命中）。
    CountryWordRole::Person
}

fn looks_like_language_base(base_lower: &str) -> bool {
    base_lower.ends_with("sk")
        || base_lower.ends_with("skt")
        || matches!(
            base_lower,
            "engelsk"
                | "tysk"
                | "fransk"
                | "spansk"
                | "svensk"
                | "dansk"
                | "norsk"
                | "finsk"
                | "russisk"
                | "kinesisk"
                | "japansk"
                | "italiensk"
                | "portugisisk"
                | "nederlandsk"
                | "islandsk"
                | "polsk"
                | "tyrkisk"
                | "arabisk"
                | "koreansk"
                | "gresk"
                | "latin"
                | "mandarin"
        )
}

fn looks_like_person_base(base_lower: &str) -> bool {
    base_lower.ends_with("mann")
        || base_lower.ends_with("kvinne")
        || base_lower.ends_with("ier")
        || base_lower.ends_with("aner")
        || base_lower.ends_with("enser")
        || base_lower.ends_with("lending")
        || base_lower.ends_with("lendinger")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::word_bank_entry::PartOfSpeech;

    fn entry(base: &str, pos: PartOfSpeech, tags: &[&str], zh: &str, en: &str) -> WordBankEntry {
        WordBankEntry {
            id: format!("id-{base}-{zh}"),
            selected: true,
            part_of_speech: pos,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            english: vec![en.to_string()],
            chinese: vec![zh.to_string()],
            base_form: base.to_string(),
            added_at: String::new(),
            verb_present_tense: None,
            verb_past_tense: None,
            verb_imperative: None,
            verb_present_participle: None,
            verb_past_participle: None,
            verb_passive_infinitive: None,
            verb_passive_present: None,
            verb_passive_past: None,
            noun_plural: None,
            noun_singular_definite: None,
            noun_plural_definite: None,
            noun_singular_definite_genitive: None,
            noun_plural_definite_genitive: None,
            noun_singular_indefinite_genitive: None,
            noun_plural_indefinite_genitive: None,
            adjective_feminine_form: None,
            adjective_neuter_form: None,
            adjective_plural_form: None,
            adjective_comparative: None,
            adjective_superlative_indefinite: None,
            adjective_superlative_definite: None,
            pronoun_object: None,
            pronoun_reflexive: None,
            pronoun_plural_subject: None,
            pronoun_plural_object: None,
            pronoun_plural_reflexive: None,
            determinative_feminine_form: None,
            determinative_neuter_form: None,
            determinative_plural_form: None,
            adverb_comparative: None,
            adverb_superlative: None,
        }
    }

    #[test]
    fn groups_shared_language_into_each_country() {
        let entries = vec![
            entry(
                "Australia",
                PartOfSpeech::Noun,
                &["country", "Australia"],
                "澳大利亚",
                "Australia",
            ),
            entry(
                "engelsk",
                PartOfSpeech::Noun,
                &["country", "Australia", "USA"],
                "英语",
                "English",
            ),
            entry(
                "USA",
                PartOfSpeech::Noun,
                &["country", "USA"],
                "美国",
                "USA",
            ),
        ];
        let ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
        let blocks = build_country_blocks(&entries, &ids);
        assert_eq!(blocks.len(), 2);
        let australia = blocks.iter().find(|b| b.country_tag == "Australia").unwrap();
        let usa = blocks.iter().find(|b| b.country_tag == "USA").unwrap();
        assert_eq!(australia.title_zh, "澳大利亚");
        assert_eq!(australia.title_en, "Australia");
        assert!(australia.entry_ids.iter().any(|id| id.contains("engelsk")));
        assert!(usa.entry_ids.iter().any(|id| id.contains("engelsk")));
    }
}
