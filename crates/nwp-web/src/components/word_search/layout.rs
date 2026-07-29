//! AI 查询主 UI：组合 Ordbok / Gemini / Google Translate 调用并分流填充结果。

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app_state::{LexiconState, UiState};
use crate::utils::i18n::tr;

use super::dictionary_search::test_google_translate_connectivity;
use super::gemini_search::test_gemini_connectivity;
use super::search_process::{SearchRequest, lexicon_base_form_overlap_message, run_word_search};
use super::{
    WORD_FORM_HINT_OPTIONS, build_bulk_csv_from_results, form_hint_label, join_pipe,
    normalize_part_of_speech, normalize_text_opt,
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
    pub set_single_tags: WriteSignal<String>,
}

#[component]
pub fn AiResearcher(
    actions: AiResearcherActions,
    single_form_state: SingleEntryFormState,
    #[prop(optional)] reset_version: Option<ReadSignal<u64>>,
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
        set_single_tags,
    } = single_form_state;

    let lang = expect_context::<UiState>().ui_language;
    let lexicon_entries = expect_context::<LexiconState>().entries;
    let (gemini_token, set_gemini_token) = signal(String::new());
    let (google_translate_token, set_google_translate_token) = signal(String::new());
    let (form_hint, set_form_hint) = signal("unknown".to_string());
    let (query_word, set_query_word) = signal(String::new());
    let (default_tags_input, set_default_tags_input) = signal(String::new());
    let (default_tags_confirmed, set_default_tags_confirmed) = signal(String::new());
    let (is_testing, set_is_testing) = signal(false);
    let (is_testing_google_translate, set_is_testing_google_translate) = signal(false);
    let (is_google_translate_verified, set_is_google_translate_verified) = signal(false);
    let (is_querying, set_is_querying) = signal(false);
    let (ordbok_only_mode, set_ordbok_only_mode) = signal(false);

    if let Some(reset_version) = reset_version {
        Effect::new(move |_| {
            let version = reset_version.get();
            if version == 0 {
                return;
            }
            set_query_word.set(String::new());
            set_form_hint.set("unknown".to_string());
        });
    }

    let confirm_default_tags = move |_| {
        let language = lang.get_untracked();
        let normalized = normalize_default_tags(&default_tags_input.get_untracked());
        set_default_tags_input.set(normalized.clone());
        set_default_tags_confirmed.set(normalized.clone());
        set_single_tags.set(normalized.clone());
        set_status.set(if normalized.is_empty() {
            tr(
                language,
                "默认标签已清空。",
                "Default tags cleared.",
            )
            .to_string()
        } else {
            format!(
                "{} {normalized}",
                tr(language, "默认标签已确认：", "Default tags confirmed:")
            )
        });
    };

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
        let default_tags = default_tags_confirmed.get_untracked();
        let ordbok_only = ordbok_only_mode.get_untracked();

        // 新查询前复位单条/多条表单，避免旧结果通过“仅填空”逻辑污染本次分流。
        set_single_pos.set("noun".to_string());
        set_single_norwegian.set(String::new());
        set_single_chinese.set(String::new());
        set_single_english.set(String::new());
        set_single_verb_present_tense.set(String::new());
        set_single_verb_past_tense.set(String::new());
        set_single_verb_imperative.set(String::new());
        set_single_verb_present_participle.set(String::new());
        set_single_verb_past_participle.set(String::new());
        set_single_verb_passive_infinitive.set(String::new());
        set_single_verb_passive_present.set(String::new());
        set_single_verb_passive_past.set(String::new());
        set_single_noun_plural.set(String::new());
        set_single_noun_singular_definite.set(String::new());
        set_single_noun_plural_definite.set(String::new());
        set_single_noun_singular_definite_genitive.set(String::new());
        set_single_noun_plural_definite_genitive.set(String::new());
        set_single_noun_singular_indefinite_genitive.set(String::new());
        set_single_noun_plural_indefinite_genitive.set(String::new());
        set_single_adjective_feminine_form.set(String::new());
        set_single_adjective_neuter_form.set(String::new());
        set_single_adjective_plural_form.set(String::new());
        set_single_adjective_comparative.set(String::new());
        set_single_adjective_superlative_indefinite.set(String::new());
        set_single_adjective_superlative_definite.set(String::new());
        set_single_pronoun_object.set(String::new());
        set_single_pronoun_reflexive.set(String::new());
        set_single_pronoun_plural_subject.set(String::new());
        set_single_pronoun_plural_object.set(String::new());
        set_single_pronoun_plural_reflexive.set(String::new());
        set_single_determinative_feminine_form.set(String::new());
        set_single_determinative_neuter_form.set(String::new());
        set_single_determinative_plural_form.set(String::new());
        set_single_adverb_comparative.set(String::new());
        set_single_adverb_superlative.set(String::new());
        set_single_tags.set(String::new());
        set_bulk_input.set(String::new());
        set_bulk_errors.set(Vec::new());
        set_bulk_success_message.set(String::new());

        set_is_querying.set(true);

        let set_is_querying = set_is_querying;
        let set_status = set_status;
        let set_single_tags = set_single_tags;
        spawn_local(async move {
            let request = SearchRequest {
                language,
                word: word.clone(),
                hint: hint.clone(),
                gemini_token,
                google_translate_token,
                google_translate_verified,
                ordbok_only,
            };
            let outcome = run_word_search(request, |message| {
                set_status.set(message);
            })
            .await;

            let (parsed_list, source_label) = match outcome {
                Ok(success) => {
                    if success.results.is_empty() {
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
                    (success.results, success.source_label)
                }
                Err(err) => {
                    set_status.set(err);
                    set_is_querying.set(false);
                    return;
                }
            };

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
                if !default_tags.trim().is_empty() {
                    set_single_tags.set(default_tags.clone());
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
                match build_bulk_csv_from_results(&parsed_list, &hint, &default_tags) {
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
            // 先复位查询按钮，让用户可继续操作；词库原型比对放到下一轮任务，不阻塞分流。
            set_is_querying.set(false);
            let lexicon_snapshot = lexicon_entries.get_untracked();
            spawn_local(async move {
                let overlap = lexicon_base_form_overlap_message(
                    language,
                    &parsed_list,
                    &lexicon_snapshot,
                );
                set_status.set(overlap);
            });
        });
    };

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">
                {move || {
                    tr(
                        lang.get(),
                        "词典优先查询（Ordbok → Google Translate → Gemini，必要时按原型二次查询）",
                        "Dictionary-first Lookup (Ordbok -> Google Translate -> Gemini; re-query by base form when needed)",
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

            <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-[auto_1fr_auto] md:items-center">
                <span class="text-sm text-slate-300">
                    {move || tr(lang.get(), "默认标签", "Default Tags")}
                </span>
                <input
                    type="text"
                    placeholder=move || {
                        tr(
                            lang.get(),
                            "例如 duolingo|time（用 | 分隔）",
                            "e.g. duolingo|time (pipe-separated)",
                        )
                    }
                    prop:value=move || default_tags_input.get()
                    on:input=move |ev| set_default_tags_input.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=confirm_default_tags
                    class="w-full rounded-lg border border-slate-600 bg-slate-800 px-4 py-2 text-sm font-medium hover:bg-slate-700 md:w-auto"
                >
                    {move || tr(lang.get(), "确认", "Confirm")}
                </button>
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

fn normalize_default_tags(raw: &str) -> String {
    raw.split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("|")
}

fn maybe_set_opt(reader: ReadSignal<String>, setter: WriteSignal<String>, value: Option<String>) {
    if let Some(value) = normalize_text_opt(value) {
        set_if_blank(reader, setter, value);
    }
}
