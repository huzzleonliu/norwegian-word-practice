//! AI 查询主 UI：组合 Ordbok / Gemini / Google Translate 调用并分流填充结果。

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::UiState;
use crate::utils::i18n::tr;

use super::dictionary_search::{
    enrich_results_with_google_translate, query_word_with_ordbok,
    test_google_translate_connectivity,
};
use super::gemini_search::{query_word_with_gemini, test_gemini_connectivity};
use super::{
    GeminiWordResult, WORD_FORM_HINT_OPTIONS, build_bulk_csv_from_results, form_hint_label,
    join_pipe, merge_word_result, normalize_part_of_speech, normalize_text_opt,
};

#[derive(Clone, Copy)]
pub struct AiResearcherActions {
    pub set_status: WriteSignal<String>,
    pub set_bulk_input: WriteSignal<String>,
    pub set_bulk_errors: WriteSignal<Vec<String>>,
    pub set_bulk_success_message: WriteSignal<String>,
}

#[derive(Clone, Copy)]
pub struct SingleEntryFormState {
    pub set_single_pos: WriteSignal<String>,
    pub single_norwegian: ReadSignal<String>,
    pub set_single_norwegian: WriteSignal<String>,
    pub single_chinese: ReadSignal<String>,
    pub set_single_chinese: WriteSignal<String>,
    pub single_english: ReadSignal<String>,
    pub set_single_english: WriteSignal<String>,
    pub single_verb_present_tense: ReadSignal<String>,
    pub set_single_verb_present_tense: WriteSignal<String>,
    pub single_verb_past_tense: ReadSignal<String>,
    pub set_single_verb_past_tense: WriteSignal<String>,
    pub single_verb_imperative: ReadSignal<String>,
    pub set_single_verb_imperative: WriteSignal<String>,
    pub single_verb_present_participle: ReadSignal<String>,
    pub set_single_verb_present_participle: WriteSignal<String>,
    pub single_verb_past_participle: ReadSignal<String>,
    pub set_single_verb_past_participle: WriteSignal<String>,
    pub single_verb_passive_infinitive: ReadSignal<String>,
    pub set_single_verb_passive_infinitive: WriteSignal<String>,
    pub single_verb_passive_present: ReadSignal<String>,
    pub set_single_verb_passive_present: WriteSignal<String>,
    pub single_verb_passive_past: ReadSignal<String>,
    pub set_single_verb_passive_past: WriteSignal<String>,
    pub single_noun_plural: ReadSignal<String>,
    pub set_single_noun_plural: WriteSignal<String>,
    pub single_noun_singular_definite: ReadSignal<String>,
    pub set_single_noun_singular_definite: WriteSignal<String>,
    pub single_noun_plural_definite: ReadSignal<String>,
    pub set_single_noun_plural_definite: WriteSignal<String>,
    pub single_noun_singular_definite_genitive: ReadSignal<String>,
    pub set_single_noun_singular_definite_genitive: WriteSignal<String>,
    pub single_noun_plural_definite_genitive: ReadSignal<String>,
    pub set_single_noun_plural_definite_genitive: WriteSignal<String>,
    pub single_noun_singular_indefinite_genitive: ReadSignal<String>,
    pub set_single_noun_singular_indefinite_genitive: WriteSignal<String>,
    pub single_noun_plural_indefinite_genitive: ReadSignal<String>,
    pub set_single_noun_plural_indefinite_genitive: WriteSignal<String>,
    pub single_adjective_feminine_form: ReadSignal<String>,
    pub set_single_adjective_feminine_form: WriteSignal<String>,
    pub single_adjective_neuter_form: ReadSignal<String>,
    pub set_single_adjective_neuter_form: WriteSignal<String>,
    pub single_adjective_plural_form: ReadSignal<String>,
    pub set_single_adjective_plural_form: WriteSignal<String>,
    pub single_adjective_comparative: ReadSignal<String>,
    pub set_single_adjective_comparative: WriteSignal<String>,
    pub single_adjective_superlative_indefinite: ReadSignal<String>,
    pub set_single_adjective_superlative_indefinite: WriteSignal<String>,
    pub single_adjective_superlative_definite: ReadSignal<String>,
    pub set_single_adjective_superlative_definite: WriteSignal<String>,
    pub single_pronoun_object: ReadSignal<String>,
    pub set_single_pronoun_object: WriteSignal<String>,
    pub single_pronoun_reflexive: ReadSignal<String>,
    pub set_single_pronoun_reflexive: WriteSignal<String>,
    pub single_pronoun_plural_subject: ReadSignal<String>,
    pub set_single_pronoun_plural_subject: WriteSignal<String>,
    pub single_pronoun_plural_object: ReadSignal<String>,
    pub set_single_pronoun_plural_object: WriteSignal<String>,
    pub single_pronoun_plural_reflexive: ReadSignal<String>,
    pub set_single_pronoun_plural_reflexive: WriteSignal<String>,
    pub single_determinative_feminine_form: ReadSignal<String>,
    pub set_single_determinative_feminine_form: WriteSignal<String>,
    pub single_determinative_neuter_form: ReadSignal<String>,
    pub set_single_determinative_neuter_form: WriteSignal<String>,
    pub single_determinative_plural_form: ReadSignal<String>,
    pub set_single_determinative_plural_form: WriteSignal<String>,
    pub single_adverb_comparative: ReadSignal<String>,
    pub set_single_adverb_comparative: WriteSignal<String>,
    pub single_adverb_superlative: ReadSignal<String>,
    pub set_single_adverb_superlative: WriteSignal<String>,
}

#[component]
pub fn AiResearcher(
    actions: AiResearcherActions,
    single_form_state: SingleEntryFormState,
) -> impl IntoView {
    let AiResearcherActions {
        set_status,
        set_bulk_input,
        set_bulk_errors,
        set_bulk_success_message,
    } = actions;
    let SingleEntryFormState {
        set_single_pos,
        single_norwegian,
        set_single_norwegian,
        single_chinese,
        set_single_chinese,
        single_english,
        set_single_english,
        single_verb_present_tense,
        set_single_verb_present_tense,
        single_verb_past_tense,
        set_single_verb_past_tense,
        single_verb_imperative,
        set_single_verb_imperative,
        single_verb_present_participle,
        set_single_verb_present_participle,
        single_verb_past_participle,
        set_single_verb_past_participle,
        single_verb_passive_infinitive,
        set_single_verb_passive_infinitive,
        single_verb_passive_present,
        set_single_verb_passive_present,
        single_verb_passive_past,
        set_single_verb_passive_past,
        single_noun_plural,
        set_single_noun_plural,
        single_noun_singular_definite,
        set_single_noun_singular_definite,
        single_noun_plural_definite,
        set_single_noun_plural_definite,
        single_noun_singular_definite_genitive,
        set_single_noun_singular_definite_genitive,
        single_noun_plural_definite_genitive,
        set_single_noun_plural_definite_genitive,
        single_noun_singular_indefinite_genitive,
        set_single_noun_singular_indefinite_genitive,
        single_noun_plural_indefinite_genitive,
        set_single_noun_plural_indefinite_genitive,
        single_adjective_feminine_form,
        set_single_adjective_feminine_form,
        single_adjective_neuter_form,
        set_single_adjective_neuter_form,
        single_adjective_plural_form,
        set_single_adjective_plural_form,
        single_adjective_comparative,
        set_single_adjective_comparative,
        single_adjective_superlative_indefinite,
        set_single_adjective_superlative_indefinite,
        single_adjective_superlative_definite,
        set_single_adjective_superlative_definite,
        single_pronoun_object,
        set_single_pronoun_object,
        single_pronoun_reflexive,
        set_single_pronoun_reflexive,
        single_pronoun_plural_subject,
        set_single_pronoun_plural_subject,
        single_pronoun_plural_object,
        set_single_pronoun_plural_object,
        single_pronoun_plural_reflexive,
        set_single_pronoun_plural_reflexive,
        single_determinative_feminine_form,
        set_single_determinative_feminine_form,
        single_determinative_neuter_form,
        set_single_determinative_neuter_form,
        single_determinative_plural_form,
        set_single_determinative_plural_form,
        single_adverb_comparative,
        set_single_adverb_comparative,
        single_adverb_superlative,
        set_single_adverb_superlative,
    } = single_form_state;

    let lang = expect_context::<UiState>().ui_language;
    let (gemini_token, set_gemini_token) = signal(String::new());
    let (google_translate_token, set_google_translate_token) = signal(String::new());
    let (form_hint, set_form_hint) = signal("unknown".to_string());
    let (query_word, set_query_word) = signal(String::new());
    let (is_testing, set_is_testing) = signal(false);
    let (is_testing_google_translate, set_is_testing_google_translate) = signal(false);
    let (is_google_translate_verified, set_is_google_translate_verified) = signal(false);
    let (is_querying, set_is_querying) = signal(false);
    let (ordbok_only_mode, set_ordbok_only_mode) = signal(false);

    // 检测 Gemini Token 可用性。
    let test_connectivity = move |_| {
        let language = lang.get_untracked();
        let token = gemini_token.get_untracked().trim().to_string();
        if token.is_empty() {
            set_status.set(
                tr(
                    language,
                    "Gemini 连接检测失败：请先输入 API Token。",
                    "Gemini connectivity check failed: please enter API token first.",
                )
                .to_string(),
            );
            return;
        }

        set_is_testing.set(true);
        let set_status = set_status;
        let set_is_testing = set_is_testing;
        spawn_local(async move {
            match test_gemini_connectivity(&token).await {
                Ok(()) => {
                    set_status.set(
                        tr(
                            language,
                            "Gemini 连通成功。",
                            "Gemini connection succeeded.",
                        )
                        .to_string(),
                    );
                }
                Err(err) => set_status.set(format!(
                    "{} {err}",
                    tr(
                        language,
                        "Gemini 连接检测失败：",
                        "Gemini connectivity check failed:"
                    )
                )),
            }
            set_is_testing.set(false);
        });
    };

    // 检测 Google Translate Token 可用性。
    let test_google_translate_connectivity = move |_| {
        let language = lang.get_untracked();
        let token = google_translate_token.get_untracked().trim().to_string();
        if token.is_empty() {
            set_is_google_translate_verified.set(false);
            set_status.set(
                tr(
                    language,
                    "Google Translate 连接检测失败：请先输入 API Token。",
                    "Google Translate connectivity check failed: please enter API token first.",
                )
                .to_string(),
            );
            return;
        }

        set_is_testing_google_translate.set(true);
        let set_status = set_status;
        let set_is_testing_google_translate = set_is_testing_google_translate;
        let set_is_google_translate_verified = set_is_google_translate_verified;
        spawn_local(async move {
            match test_google_translate_connectivity(&token).await {
                Ok(reply) if !reply.trim().is_empty() => {
                    set_is_google_translate_verified.set(true);
                    set_status.set(format!(
                        "{} {}",
                        tr(
                            language,
                            "Google Translate 连通成功，示例翻译：",
                            "Google Translate connection succeeded, sample:",
                        ),
                        reply
                    ));
                }
                Ok(_) => {
                    set_is_google_translate_verified.set(false);
                    set_status.set(
                        tr(
                            language,
                            "Google Translate 有响应，但返回内容为空。",
                            "Google Translate responded but returned empty text.",
                        )
                        .to_string(),
                    );
                }
                Err(err) => {
                    set_is_google_translate_verified.set(false);
                    set_status.set(format!(
                        "{} {err}",
                        tr(
                            language,
                            "Google Translate 连接检测失败：",
                            "Google Translate connectivity check failed:",
                        )
                    ));
                }
            }
            set_is_testing_google_translate.set(false);
        });
    };

    // 查询主流程：优先 Ordbok，按配置回填翻译并在必要时回退 Gemini。
    let query_forms = move |_| {
        let language = lang.get_untracked();
        let gemini_token = gemini_token.get_untracked().trim().to_string();
        let google_translate_token = google_translate_token.get_untracked().trim().to_string();
        let google_translate_verified = is_google_translate_verified.get_untracked();
        let word = query_word.get_untracked().trim().to_string();
        if word.is_empty() {
            set_status.set(
                tr(
                    language,
                    "查询失败：请输入要查询的词。",
                    "Query failed: please enter a target word.",
                )
                .to_string(),
            );
            return;
        }
        let hint = form_hint.get_untracked();
        let ordbok_only = ordbok_only_mode.get_untracked();
        set_is_querying.set(true);

        let set_is_querying = set_is_querying;
        let set_status = set_status;
        spawn_local(async move {
            let (parsed_list, source_label) = match query_word_with_ordbok(&word, &hint).await {
                Ok(mut results) if !results.is_empty() => {
                    let mut source_label = tr(language, "Ordbok API", "Ordbok API").to_string();
                    let can_use_google_translate =
                        !google_translate_token.is_empty() && google_translate_verified;
                    let mut used_google_translate = false;

                    if can_use_google_translate {
                        match enrich_results_with_google_translate(
                            &mut results,
                            &google_translate_token,
                        )
                        .await
                        {
                            Ok(()) => {
                                used_google_translate = true;
                                source_label = tr(
                                    language,
                                    "Ordbok API + Google Translate",
                                    "Ordbok API + Google Translate",
                                )
                                .to_string();
                            }
                            Err(err) => {
                                set_status.set(format!(
                                    "{} {err}",
                                    tr(
                                        language,
                                        "提示：Google Translate 回填失败，改用 Gemini 补全。",
                                        "Notice: Google Translate fill failed; falling back to Gemini fill.",
                                    )
                                ));
                            }
                        }
                    }

                    let needs_degree_fill = results_need_adjective_degree_fill(&results);
                    let should_call_gemini = !gemini_token.is_empty()
                        && !ordbok_only
                        && (!used_google_translate || needs_degree_fill);

                    if should_call_gemini {
                        match query_word_with_gemini(&gemini_token, &word, &hint).await {
                            Ok(gemini_results) => {
                                fill_missing_fields_from_gemini(
                                    &mut results,
                                    &gemini_results,
                                    &hint,
                                );
                                source_label = if used_google_translate {
                                    tr(
                                        language,
                                        "Ordbok API + Google Translate + Gemini（补缺词形）",
                                        "Ordbok API + Google Translate + Gemini (missing forms)",
                                    )
                                    .to_string()
                                } else {
                                    tr(
                                        language,
                                        "Ordbok API + Gemini（补中英与缺词形）",
                                        "Ordbok API + Gemini (translations and missing forms)",
                                    )
                                    .to_string()
                                };
                            }
                            Err(err) => {
                                set_status.set(format!(
                                    "{} {err}",
                                    tr(
                                        language,
                                        "提示：Gemini 补全失败，保留词典结果。",
                                        "Notice: Gemini fill failed; keeping dictionary result.",
                                    )
                                ));
                            }
                        }
                    } else if !used_google_translate {
                        source_label = tr(
                            language,
                            "Ordbok API（中英留空：未通过 Google 检测且未提供 Gemini Token）",
                            "Ordbok API (Chinese/English empty: Google not verified and Gemini token missing)",
                        )
                        .to_string();
                    }

                    (results, source_label)
                }
                Ok(_) => {
                    if ordbok_only {
                        set_status.set(
                            tr(
                                language,
                                "Ordbok API 未命中，且已开启“仅 Ordbok”模式，不回退 Gemini。",
                                "No Ordbok match and \"Ordbok-only\" mode is enabled; Gemini fallback skipped.",
                            )
                            .to_string(),
                        );
                        set_is_querying.set(false);
                        return;
                    }
                    if gemini_token.is_empty() {
                        set_status.set(
                            tr(
                                language,
                                "Ordbok API 未命中，且未提供 Gemini Token，无法回退 AI 查询。",
                                "No Ordbok match and Gemini token is missing; cannot fallback to AI.",
                            )
                            .to_string(),
                        );
                        set_is_querying.set(false);
                        return;
                    }
                    match query_word_with_gemini(&gemini_token, &word, &hint).await {
                        Ok(results) => (
                            results,
                            tr(
                                language,
                                "Gemini（Ordbok 未命中后回退）",
                                "Gemini (fallback after no Ordbok match)",
                            )
                            .to_string(),
                        ),
                        Err(err) => {
                            set_status.set(format!(
                                "{} {err}",
                                tr(
                                    language,
                                    "查询失败：Ordbok 未命中，Gemini 回退也失败。",
                                    "Query failed: no Ordbok match and Gemini fallback also failed.",
                                )
                            ));
                            set_is_querying.set(false);
                            return;
                        }
                    }
                }
                Err(ordbok_err) => {
                    if ordbok_only {
                        set_status.set(format!(
                            "{} {ordbok_err}",
                            tr(
                                language,
                                "查询失败：Ordbok API 请求失败，且已开启“仅 Ordbok”模式。",
                                "Query failed: Ordbok API request failed and \"Ordbok-only\" mode is enabled.",
                            )
                        ));
                        set_is_querying.set(false);
                        return;
                    }
                    if gemini_token.is_empty() {
                        set_status.set(format!(
                            "{} {ordbok_err}",
                            tr(
                                language,
                                "查询失败：Ordbok API 请求失败，且未提供 Gemini Token。",
                                "Query failed: Ordbok API request failed and Gemini token is missing.",
                            )
                        ));
                        set_is_querying.set(false);
                        return;
                    }
                    match query_word_with_gemini(&gemini_token, &word, &hint).await {
                        Ok(results) => (
                            results,
                            format!(
                                "{} ({ordbok_err})",
                                tr(
                                    language,
                                    "Gemini（Ordbok 失败后回退）",
                                    "Gemini (fallback after Ordbok error)",
                                )
                            ),
                        ),
                        Err(ai_err) => {
                            set_status.set(format!(
                                "{} {ordbok_err}；{} {ai_err}",
                                tr(
                                    language,
                                    "查询失败：Ordbok API 请求失败：",
                                    "Query failed: Ordbok API request failed:",
                                ),
                                tr(
                                    language,
                                    "且 Gemini 回退失败：",
                                    "and Gemini fallback failed:",
                                )
                            ));
                            set_is_querying.set(false);
                            return;
                        }
                    }
                }
            };

            if parsed_list.is_empty() {
                set_status.set(
                    tr(
                        language,
                        "查询失败：没有可用结果。",
                        "Query failed: no usable result returned.",
                    )
                    .to_string(),
                );
                set_is_querying.set(false);
                return;
            }

            if parsed_list.len() == 1 {
                let parsed = parsed_list.first().cloned().unwrap_or_default();
                let pos = normalize_part_of_speech(parsed.part_of_speech.clone(), &hint);
                set_single_pos.set(pos);

                if let Some(v) = normalize_text_opt(parsed.base_form) {
                    set_if_blank(single_norwegian, set_single_norwegian, v);
                }
                if let Some(v) = join_pipe(parsed.chinese) {
                    set_if_blank(single_chinese, set_single_chinese, v);
                }
                if let Some(v) = join_pipe(parsed.english) {
                    set_if_blank(single_english, set_single_english, v);
                }

                maybe_set_opt(
                    single_verb_present_tense,
                    set_single_verb_present_tense,
                    parsed.verb_present_tense,
                );
                maybe_set_opt(
                    single_verb_past_tense,
                    set_single_verb_past_tense,
                    parsed.verb_past_tense,
                );
                maybe_set_opt(
                    single_verb_imperative,
                    set_single_verb_imperative,
                    parsed.verb_imperative,
                );
                maybe_set_opt(
                    single_verb_present_participle,
                    set_single_verb_present_participle,
                    parsed.verb_present_participle,
                );
                maybe_set_opt(
                    single_verb_past_participle,
                    set_single_verb_past_participle,
                    parsed.verb_past_participle,
                );
                maybe_set_opt(
                    single_verb_passive_infinitive,
                    set_single_verb_passive_infinitive,
                    parsed.verb_passive_infinitive,
                );
                maybe_set_opt(
                    single_verb_passive_present,
                    set_single_verb_passive_present,
                    parsed.verb_passive_present,
                );
                maybe_set_opt(
                    single_verb_passive_past,
                    set_single_verb_passive_past,
                    parsed.verb_passive_past,
                );
                maybe_set_opt(
                    single_noun_plural,
                    set_single_noun_plural,
                    parsed.noun_plural,
                );
                maybe_set_opt(
                    single_noun_singular_definite,
                    set_single_noun_singular_definite,
                    parsed.noun_singular_definite,
                );
                maybe_set_opt(
                    single_noun_plural_definite,
                    set_single_noun_plural_definite,
                    parsed.noun_plural_definite,
                );
                maybe_set_opt(
                    single_noun_singular_definite_genitive,
                    set_single_noun_singular_definite_genitive,
                    parsed.noun_singular_definite_genitive,
                );
                maybe_set_opt(
                    single_noun_plural_definite_genitive,
                    set_single_noun_plural_definite_genitive,
                    parsed.noun_plural_definite_genitive,
                );
                maybe_set_opt(
                    single_noun_singular_indefinite_genitive,
                    set_single_noun_singular_indefinite_genitive,
                    parsed.noun_singular_indefinite_genitive,
                );
                maybe_set_opt(
                    single_noun_plural_indefinite_genitive,
                    set_single_noun_plural_indefinite_genitive,
                    parsed.noun_plural_indefinite_genitive,
                );
                maybe_set_opt(
                    single_adjective_feminine_form,
                    set_single_adjective_feminine_form,
                    parsed.adjective_feminine_form,
                );
                maybe_set_opt(
                    single_adjective_neuter_form,
                    set_single_adjective_neuter_form,
                    parsed.adjective_neuter_form,
                );
                maybe_set_opt(
                    single_adjective_plural_form,
                    set_single_adjective_plural_form,
                    parsed.adjective_plural_form,
                );
                maybe_set_opt(
                    single_adjective_comparative,
                    set_single_adjective_comparative,
                    parsed.adjective_comparative,
                );
                maybe_set_opt(
                    single_adjective_superlative_indefinite,
                    set_single_adjective_superlative_indefinite,
                    parsed.adjective_superlative_indefinite,
                );
                maybe_set_opt(
                    single_adjective_superlative_definite,
                    set_single_adjective_superlative_definite,
                    parsed.adjective_superlative_definite,
                );
                maybe_set_opt(
                    single_pronoun_object,
                    set_single_pronoun_object,
                    parsed.pronoun_object,
                );
                maybe_set_opt(
                    single_pronoun_reflexive,
                    set_single_pronoun_reflexive,
                    parsed.pronoun_reflexive,
                );
                maybe_set_opt(
                    single_pronoun_plural_subject,
                    set_single_pronoun_plural_subject,
                    parsed.pronoun_plural_subject,
                );
                maybe_set_opt(
                    single_pronoun_plural_object,
                    set_single_pronoun_plural_object,
                    parsed.pronoun_plural_object,
                );
                maybe_set_opt(
                    single_pronoun_plural_reflexive,
                    set_single_pronoun_plural_reflexive,
                    parsed.pronoun_plural_reflexive,
                );
                maybe_set_opt(
                    single_determinative_feminine_form,
                    set_single_determinative_feminine_form,
                    parsed.determinative_feminine_form,
                );
                maybe_set_opt(
                    single_determinative_neuter_form,
                    set_single_determinative_neuter_form,
                    parsed.determinative_neuter_form,
                );
                maybe_set_opt(
                    single_determinative_plural_form,
                    set_single_determinative_plural_form,
                    parsed.determinative_plural_form,
                );
                maybe_set_opt(
                    single_adverb_comparative,
                    set_single_adverb_comparative,
                    parsed.adverb_comparative,
                );
                maybe_set_opt(
                    single_adverb_superlative,
                    set_single_adverb_superlative,
                    parsed.adverb_superlative,
                );

                set_status.set(format!(
                    "{source_label} \"{word}\" {}",
                    tr(
                        language,
                        "查询成功：已填充单条结果。",
                        "Query succeeded: single-entry result filled."
                    )
                ));
            } else {
                match build_bulk_csv_from_results(&parsed_list, &hint) {
                    Ok(csv_text) => {
                        set_bulk_input.set(csv_text);
                        set_bulk_errors.set(Vec::new());
                        set_bulk_success_message.set(String::new());
                        set_status.set(format!(
                            "{source_label} {} {} {}",
                            tr(language, "返回", "returned"),
                            parsed_list.len(),
                            tr(
                                language,
                                "条结果，已写入“多条添加”输入框。",
                                "results and wrote them into bulk-add input.",
                            )
                        ));
                    }
                    Err(err) => {
                        set_status.set(format!(
                            "{} {err}",
                            tr(
                                language,
                                "查询失败：多条结果转 CSV 失败。",
                                "Query failed: multi-result CSV conversion failed.",
                            )
                        ));
                    }
                }
            }
            set_is_querying.set(false);
        });
    };

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">
                {move || {
                    tr(
                        lang.get(),
                        "词典优先查询（Ordbok API → Gemini）",
                        "Dictionary-first Lookup (Ordbok API -> Gemini)",
                    )
                }}
            </h2>

            <div class="grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto]">
                <input
                    type="password"
                    placeholder=move || {
                        tr(
                            lang.get(),
                            "输入 Gemini API Token（仅 Ordbok 未命中/失败时回退）",
                            "Gemini API token (used only when Ordbok misses/fails)",
                        )
                    }
                    prop:value=move || gemini_token.get()
                    on:input=move |ev| set_gemini_token.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=test_connectivity
                    disabled=move || is_testing.get()
                    class="w-full rounded-lg border border-cyan-700 bg-cyan-700 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-600 disabled:cursor-not-allowed disabled:opacity-60 md:w-auto"
                >
                    {move || {
                        if is_testing.get() {
                            tr(lang.get(), "检测中...", "Checking...")
                        } else {
                            tr(lang.get(), "检测连通", "Test Connectivity")
                        }
                    }}
                </button>
            </div>

            <div class="mt-2 grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto]">
                <input
                    type="password"
                    placeholder=move || {
                        tr(
                            lang.get(),
                            "输入 Google Translate API Token（Ordbok 命中后用于回填中英）",
                            "Google Translate API token (fill Chinese/English after Ordbok match)",
                        )
                    }
                    prop:value=move || google_translate_token.get()
                    on:input=move |ev| {
                        set_google_translate_token.set(event_target_value(&ev));
                        set_is_google_translate_verified.set(false);
                    }
                    class="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=test_google_translate_connectivity
                    disabled=move || is_testing_google_translate.get()
                    class="w-full rounded-lg border border-sky-700 bg-sky-700 px-4 py-2 text-sm font-medium text-white hover:bg-sky-600 disabled:cursor-not-allowed disabled:opacity-60 md:w-auto"
                >
                    {move || {
                        if is_testing_google_translate.get() {
                            tr(lang.get(), "检测中...", "Checking...")
                        } else {
                            tr(lang.get(), "检测翻译连接", "Test Translate Connectivity")
                        }
                    }}
                </button>
            </div>

            <div class="mt-2">
                <label class="inline-flex items-center gap-2 text-xs text-slate-300">
                    <input
                        type="checkbox"
                        prop:checked=move || ordbok_only_mode.get()
                        on:change=move |ev| set_ordbok_only_mode.set(event_target_checked(&ev))
                    />
                    <span>
                        {move || {
                            tr(
                                lang.get(),
                                "仅使用 Ordbok API（不回退 Gemini）",
                                "Ordbok only (disable Gemini fallback)",
                            )
                        }}
                    </span>
                </label>
            </div>

            <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-[220px_1fr_auto]">
                <select
                    prop:value=move || form_hint.get()
                    on:change=move |ev| set_form_hint.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                >
                    {WORD_FORM_HINT_OPTIONS
                        .iter()
                        .map(|value| {
                            view! {
                                <option value=*value>
                                    {move || form_hint_label(lang.get(), value)}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
                <input
                    type="text"
                    placeholder=move || tr(lang.get(), "输入单词", "Enter word")
                    prop:value=move || query_word.get()
                    on:input=move |ev| set_query_word.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=query_forms
                    disabled=move || is_querying.get()
                    class="w-full rounded-lg border border-emerald-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-60 md:w-auto"
                >
                    {move || {
                        if is_querying.get() {
                            tr(lang.get(), "查询中...", "Querying...")
                        } else {
                            tr(lang.get(), "查询并分流", "Query and Route")
                        }
                    }}
                </button>
            </div>
        </section>
    }
}

fn set_if_blank(reader: ReadSignal<String>, setter: WriteSignal<String>, value: String) {
    if reader.get_untracked().trim().is_empty() {
        setter.set(value);
    }
}

fn maybe_set_opt(reader: ReadSignal<String>, setter: WriteSignal<String>, value: Option<String>) {
    if let Some(value) = normalize_text_opt(value) {
        set_if_blank(reader, setter, value);
    }
}

fn fill_missing_fields_from_gemini(
    dictionary_results: &mut [GeminiWordResult],
    gemini_results: &[GeminiWordResult],
    hint: &str,
) {
    for item in dictionary_results.iter_mut() {
        let Some(source) = find_best_translation_source(item, gemini_results, hint) else {
            continue;
        };
        merge_word_result(item, source.clone());
    }
}

fn results_need_adjective_degree_fill(results: &[GeminiWordResult]) -> bool {
    results.iter().any(|item| {
        let pos = normalize_part_of_speech(item.part_of_speech.clone(), "adjective");
        pos == "adjective"
            && (item.adjective_comparative.is_none()
                || item.adjective_superlative_indefinite.is_none()
                || item.adjective_superlative_definite.is_none())
    })
}

fn find_best_translation_source<'a>(
    target: &GeminiWordResult,
    candidates: &'a [GeminiWordResult],
    hint: &str,
) -> Option<&'a GeminiWordResult> {
    let target_base = normalize_text_opt(target.base_form.clone()).unwrap_or_default();
    let target_pos = normalize_part_of_speech(target.part_of_speech.clone(), hint);

    candidates
        .iter()
        .find(|item| {
            normalize_text_opt(item.base_form.clone())
                .unwrap_or_default()
                .eq_ignore_ascii_case(&target_base)
                && normalize_part_of_speech(item.part_of_speech.clone(), hint) == target_pos
        })
        .or_else(|| {
            candidates.iter().find(|item| {
                normalize_text_opt(item.base_form.clone())
                    .unwrap_or_default()
                    .eq_ignore_ascii_case(&target_base)
            })
        })
        .or_else(|| (candidates.len() == 1).then(|| &candidates[0]))
}
