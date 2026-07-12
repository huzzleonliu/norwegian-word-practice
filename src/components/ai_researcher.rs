use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::Value;

use crate::app_state::WordBankState;
use crate::components::lexicon_browser::serialize_word_bank_csv;
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};
use crate::utils::i18n::tr;

const WORD_FORM_HINT_OPTIONS: [(&str, &str); 23] = [
    ("unknown", "未知"),
    ("noun-baseform", "noun-baseform"),
    ("noun_plural", "noun_plural"),
    ("noun_singular_definite", "noun_singular_definite"),
    ("noun_plural_definite", "noun_plural_definite"),
    ("verb-baseform", "verb-baseform"),
    ("verb_present_tense", "verb_present_tense"),
    ("verb_past_tense", "verb_past_tense"),
    ("verb_imperative", "verb_imperative"),
    ("adjective-baseform", "adjective-baseform"),
    ("adjective_neuter_form", "adjective_neuter_form"),
    ("adjective_plural_form", "adjective_plural_form"),
    ("adjective_comparative", "adjective_comparative"),
    (
        "adjective_superlative_indefinite",
        "adjective_superlative_indefinite",
    ),
    (
        "adjective_superlative_definite",
        "adjective_superlative_definite",
    ),
    ("adverb-baseform", "adverb-baseform"),
    ("adverb_comparative", "adverb_comparative"),
    ("adverb_superlative", "adverb_superlative"),
    ("cardinal_number-baseform", "cardinal_number-baseform"),
    ("ordinal_number-baseform", "ordinal_number-baseform"),
    ("month-baseform", "month-baseform"),
    ("pronoun-baseform", "pronoun-baseform"),
    ("interrogative-baseform", "interrogative-baseform"),
];

const GEMINI_MODEL_CANDIDATES: [&str; 3] =
    ["gemini-2.5-flash", "gemini-2.5-pro", "gemini-1.5-flash"];
const ORDBOK_GRAPHQL_ENDPOINT: &str = "https://api.ordbokapi.org/graphql";
const ORDBOK_LOOKUP_QUERY: &str = r#"
query LookUp($word: String!) {
  suggestions(word: $word) {
    exact {
      word
      articles {
        wordClass
        lemmas {
          lemma
          paradigms {
            inflections {
              tags
              wordForm
            }
          }
        }
        flatDefinitions {
          content {
            textContent
          }
        }
      }
    }
  }
}
"#;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct GeminiWordResult {
    part_of_speech: Option<String>,
    base_form: Option<String>,
    chinese: Vec<String>,
    english: Vec<String>,
    #[serde(alias = "present_tense")]
    verb_present_tense: Option<String>,
    #[serde(alias = "past_tense")]
    verb_past_tense: Option<String>,
    #[serde(alias = "imperative")]
    verb_imperative: Option<String>,
    #[serde(alias = "plural")]
    noun_plural: Option<String>,
    #[serde(alias = "singular_definite")]
    noun_singular_definite: Option<String>,
    #[serde(alias = "plural_definite")]
    noun_plural_definite: Option<String>,
    #[serde(alias = "neuter_form")]
    adjective_neuter_form: Option<String>,
    #[serde(alias = "plural_form")]
    adjective_plural_form: Option<String>,
    adjective_comparative: Option<String>,
    adjective_superlative_indefinite: Option<String>,
    adjective_superlative_definite: Option<String>,
    adverb_comparative: Option<String>,
    adverb_superlative: Option<String>,
}

impl TryFrom<(&GeminiWordResult, &str)> for WordBankEntry {
    type Error = String;

    fn try_from((value, hint): (&GeminiWordResult, &str)) -> Result<Self, Self::Error> {
        Ok(Self {
            id: String::new(),
            selected: true,
            part_of_speech: normalize_part_of_speech_enum(value.part_of_speech.clone(), hint),
            tags: Vec::new(),
            english: normalize_text_list(&value.english),
            chinese: normalize_text_list(&value.chinese),
            base_form: normalize_text_opt(value.base_form.clone()).unwrap_or_default(),
            verb_present_tense: normalize_text_opt(value.verb_present_tense.clone()),
            verb_past_tense: normalize_text_opt(value.verb_past_tense.clone()),
            verb_imperative: normalize_text_opt(value.verb_imperative.clone()),
            noun_plural: normalize_text_opt(value.noun_plural.clone()),
            noun_singular_definite: normalize_text_opt(value.noun_singular_definite.clone()),
            noun_plural_definite: normalize_text_opt(value.noun_plural_definite.clone()),
            adjective_neuter_form: normalize_text_opt(value.adjective_neuter_form.clone()),
            adjective_plural_form: normalize_text_opt(value.adjective_plural_form.clone()),
            adjective_comparative: normalize_text_opt(value.adjective_comparative.clone()),
            adjective_superlative_indefinite: normalize_text_opt(
                value.adjective_superlative_indefinite.clone(),
            ),
            adjective_superlative_definite: normalize_text_opt(
                value.adjective_superlative_definite.clone(),
            ),
            adverb_comparative: normalize_text_opt(value.adverb_comparative.clone()),
            adverb_superlative: normalize_text_opt(value.adverb_superlative.clone()),
        })
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrdbokGraphQlResponse {
    data: Option<OrdbokGraphQlData>,
    #[serde(default)]
    errors: Vec<OrdbokGraphQlError>,
}

#[derive(Debug, Deserialize)]
struct OrdbokGraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct OrdbokGraphQlData {
    suggestions: OrdbokSuggestions,
}

#[derive(Debug, Deserialize)]
struct OrdbokSuggestions {
    #[serde(default)]
    exact: Vec<OrdbokExactSuggestion>,
}

#[derive(Debug, Deserialize)]
struct OrdbokExactSuggestion {
    word: String,
    #[serde(default)]
    articles: Vec<OrdbokArticle>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrdbokArticle {
    word_class: Option<String>,
    #[serde(default)]
    lemmas: Vec<OrdbokLemma>,
    #[serde(default)]
    flat_definitions: Vec<OrdbokFlatDefinition>,
}

#[derive(Debug, Deserialize)]
struct OrdbokLemma {
    lemma: String,
    #[serde(default)]
    paradigms: Vec<OrdbokParadigm>,
}

#[derive(Debug, Deserialize)]
struct OrdbokParadigm {
    #[serde(default)]
    inflections: Vec<OrdbokInflection>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrdbokInflection {
    #[serde(default)]
    tags: Vec<String>,
    word_form: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrdbokFlatDefinition {
    #[serde(default)]
    content: Vec<OrdbokRichContentItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrdbokRichContentItem {
    text_content: Option<String>,
}

#[component]
pub fn AiResearcher(
    set_status: WriteSignal<String>,
    set_bulk_input: WriteSignal<String>,
    set_bulk_errors: WriteSignal<Vec<String>>,
    set_bulk_success_message: WriteSignal<String>,
    set_single_pos: WriteSignal<String>,
    single_norwegian: ReadSignal<String>,
    set_single_norwegian: WriteSignal<String>,
    single_chinese: ReadSignal<String>,
    set_single_chinese: WriteSignal<String>,
    single_english: ReadSignal<String>,
    set_single_english: WriteSignal<String>,
    single_verb_present_tense: ReadSignal<String>,
    set_single_verb_present_tense: WriteSignal<String>,
    single_verb_past_tense: ReadSignal<String>,
    set_single_verb_past_tense: WriteSignal<String>,
    single_verb_imperative: ReadSignal<String>,
    set_single_verb_imperative: WriteSignal<String>,
    single_noun_plural: ReadSignal<String>,
    set_single_noun_plural: WriteSignal<String>,
    single_noun_singular_definite: ReadSignal<String>,
    set_single_noun_singular_definite: WriteSignal<String>,
    single_noun_plural_definite: ReadSignal<String>,
    set_single_noun_plural_definite: WriteSignal<String>,
    single_adjective_neuter_form: ReadSignal<String>,
    set_single_adjective_neuter_form: WriteSignal<String>,
    single_adjective_plural_form: ReadSignal<String>,
    set_single_adjective_plural_form: WriteSignal<String>,
    single_adjective_comparative: ReadSignal<String>,
    set_single_adjective_comparative: WriteSignal<String>,
    single_adjective_superlative_indefinite: ReadSignal<String>,
    set_single_adjective_superlative_indefinite: WriteSignal<String>,
    single_adjective_superlative_definite: ReadSignal<String>,
    set_single_adjective_superlative_definite: WriteSignal<String>,
    single_adverb_comparative: ReadSignal<String>,
    set_single_adverb_comparative: WriteSignal<String>,
    single_adverb_superlative: ReadSignal<String>,
    set_single_adverb_superlative: WriteSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<WordBankState>().ui_language;
    let (gemini_token, set_gemini_token) = signal(String::new());
    let (form_hint, set_form_hint) = signal("unknown".to_string());
    let (query_word, set_query_word) = signal(String::new());
    let (is_testing, set_is_testing) = signal(false);
    let (is_querying, set_is_querying) = signal(false);
    let (ordbok_only_mode, set_ordbok_only_mode) = signal(false);

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
            match call_gemini_text(&token, "Reply with exactly: CONNECTED").await {
                Ok(reply) if reply.to_uppercase().contains("CONNECTED") => {
                    set_status.set(
                        tr(language, "Gemini 连通成功。", "Gemini connection succeeded.").to_string(),
                    );
                }
                Ok(reply) => set_status.set(format!(
                    "{} {reply}",
                    tr(
                        language,
                        "Gemini 有响应，但检测返回异常：",
                        "Gemini responded, but health check reply was unexpected:",
                    )
                )),
                Err(err) => set_status.set(format!(
                    "{} {err}",
                    tr(language, "Gemini 连接检测失败：", "Gemini connectivity check failed:")
                )),
            }
            set_is_testing.set(false);
        });
    };

    let query_forms = move |_| {
        let language = lang.get_untracked();
        let token = gemini_token.get_untracked().trim().to_string();
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
                Ok(results) if !results.is_empty() => {
                    (results, tr(language, "Ordbok API", "Ordbok API").to_string())
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
                    if token.is_empty() {
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
                    match query_word_with_gemini(&token, &word, &hint).await {
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
                    if token.is_empty() {
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
                    match query_word_with_gemini(&token, &word, &hint).await {
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
                maybe_set_opt(single_noun_plural, set_single_noun_plural, parsed.noun_plural);
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
                    tr(language, "查询成功：已填充单条结果。", "Query succeeded: single-entry result filled.")
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
                        .map(|(value, label)| {
                            view! {
                                <option value=*value>
                                    {move || {
                                        if *value == "unknown" {
                                            tr(lang.get(), "未知", "Unknown")
                                        } else {
                                            *label
                                        }
                                    }}
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

fn normalize_text_opt(value: Option<String>) -> Option<String> {
    let value = value.unwrap_or_default();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn join_pipe(values: Vec<String>) -> Option<String> {
    let list = values
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if list.is_empty() {
        None
    } else {
        Some(list.join(" | "))
    }
}

fn normalize_text_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn hint_to_part_of_speech(hint: &str) -> Option<PartOfSpeech> {
    let prefix = hint.split(['-', '_']).next().unwrap_or_default().trim();
    PartOfSpeech::from_key(prefix).ok()
}

fn normalize_part_of_speech(raw: Option<String>, hint: &str) -> String {
    normalize_part_of_speech_enum(raw, hint)
        .as_key()
        .to_string()
}

fn normalize_part_of_speech_enum(raw: Option<String>, hint: &str) -> PartOfSpeech {
    if let Some(pos) = normalize_text_opt(raw) {
        if let Ok(parsed) = PartOfSpeech::from_key(&pos) {
            return parsed;
        }
    }
    hint_to_part_of_speech(hint).unwrap_or(PartOfSpeech::Noun)
}

fn build_bulk_csv_from_results(results: &[GeminiWordResult], hint: &str) -> Result<String, String> {
    let entries = results
        .iter()
        .map(|item| WordBankEntry::try_from((item, hint)))
        .collect::<Result<Vec<_>, _>>()?;
    serialize_word_bank_csv(&entries)
}

fn build_research_prompt(word: &str, hint: &str) -> String {
    format!(
        "你是挪威语词形助手。已知用户输入单词是：{word}；它的已知形式提示是：{hint}。\n\
请输出该词在词库中的完整信息，并且只返回 JSON（不要 markdown，不要解释）。\n\
如果存在多个合理词条（例如同形异义、不同词性），请返回 JSON 数组。\n\
如果只有一个词条，可返回单个 JSON 对象。\n\
对象结构严格如下（字段可为 null 或空数组）：\n\
{{\n\
  \"part_of_speech\": \"noun|verb|adjective|adverb|cardinal_number|ordinal_number|month|pronoun|interrogative\",\n\
  \"base_form\": \"\",\n\
  \"chinese\": [],\n\
  \"english\": [],\n\
  \"verb_present_tense\": null,\n\
  \"verb_past_tense\": null,\n\
  \"verb_imperative\": null,\n\
  \"noun_plural\": null,\n\
  \"noun_singular_definite\": null,\n\
  \"noun_plural_definite\": null,\n\
  \"adjective_neuter_form\": null,\n\
  \"adjective_plural_form\": null,\n\
  \"adjective_comparative\": null,\n\
  \"adjective_superlative_indefinite\": null,\n\
  \"adjective_superlative_definite\": null,\n\
  \"adverb_comparative\": null,\n\
  \"adverb_superlative\": null\n\
}}"
    )
}

async fn query_word_with_gemini(token: &str, word: &str, hint: &str) -> Result<Vec<GeminiWordResult>, String> {
    let prompt = build_research_prompt(word, hint);
    let raw_text = call_gemini_text(token, &prompt).await?;
    parse_gemini_word_results(&raw_text)
        .map_err(|err| format!("Gemini response parse failed: {err}"))
}

async fn query_word_with_ordbok(word: &str, hint: &str) -> Result<Vec<GeminiWordResult>, String> {
    let payload_body = serde_json::json!({
        "query": ORDBOK_LOOKUP_QUERY,
        "variables": {
            "word": word
        }
    })
    .to_string();
    let request = Request::post(ORDBOK_GRAPHQL_ENDPOINT)
        .header("Content-Type", "application/json")
        .body(payload_body)
        .map_err(|err| format!("Ordbok request build failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("Ordbok request failed: {err}"))?;
    if !response.ok() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(failed to read response body)".to_string());
        return Err(format!("Ordbok request failed: HTTP {status}: {body}"));
    }

    let parsed = response
        .json::<OrdbokGraphQlResponse>()
        .await
        .map_err(|err| format!("Ordbok response parse failed: {err}"))?;
    if !parsed.errors.is_empty() {
        let message = parsed
            .errors
            .iter()
            .map(|err| err.message.clone())
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(format!("Ordbok GraphQL error: {message}"));
    }

    let Some(data) = parsed.data else {
        return Ok(Vec::new());
    };
    Ok(parse_ordbok_results(data, hint))
}

fn parse_ordbok_results(data: OrdbokGraphQlData, hint: &str) -> Vec<GeminiWordResult> {
    let mut merged = Vec::<GeminiWordResult>::new();
    for exact in data.suggestions.exact {
        for article in exact.articles {
            if let Some(item) = convert_ordbok_article(&exact.word, &article, hint) {
                merge_or_insert_word_result(&mut merged, item);
            }
        }
    }

    let Some(expected_pos) = hint_to_part_of_speech(hint).map(|pos| pos.as_key().to_string()) else {
        return merged;
    };
    let filtered = merged
        .iter()
        .filter(|item| normalize_part_of_speech(item.part_of_speech.clone(), hint) == expected_pos)
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() { merged } else { filtered }
}

fn convert_ordbok_article(
    exact_word: &str,
    article: &OrdbokArticle,
    hint: &str,
) -> Option<GeminiWordResult> {
    let base_form = article
        .lemmas
        .first()
        .map(|lemma| lemma.lemma.clone())
        .or_else(|| normalize_text_opt(Some(exact_word.to_string())))
        .and_then(|value| normalize_text_opt(Some(value)))?;

    let part_of_speech = map_ordbok_word_class_to_key(article.word_class.as_deref(), hint)
        .map(|value| value.to_string())
        .or_else(|| hint_to_part_of_speech(hint).map(|pos| pos.as_key().to_string()));
    let normalized_pos = normalize_part_of_speech(part_of_speech.clone(), hint);
    let inflections = collect_article_inflections(article);

    let mut result = GeminiWordResult {
        part_of_speech,
        base_form: Some(base_form),
        chinese: Vec::new(),
        english: extract_article_definition_texts(article),
        verb_present_tense: None,
        verb_past_tense: None,
        verb_imperative: None,
        noun_plural: None,
        noun_singular_definite: None,
        noun_plural_definite: None,
        adjective_neuter_form: None,
        adjective_plural_form: None,
        adjective_comparative: None,
        adjective_superlative_indefinite: None,
        adjective_superlative_definite: None,
        adverb_comparative: None,
        adverb_superlative: None,
    };

    match normalized_pos.as_str() {
        "verb" => {
            result.verb_present_tense =
                pick_inflection_form(&inflections, &["Presens"], &["Passiv", "SPassiv"]);
            result.verb_past_tense =
                pick_inflection_form(&inflections, &["Preteritum"], &["Passiv", "SPassiv"]);
            result.verb_imperative =
                pick_inflection_form(&inflections, &["Imperativ"], &["Passiv", "SPassiv"]);
        }
        "noun" => {
            result.noun_plural = pick_inflection_form(&inflections, &["Fleirtal"], &["Bestemt"]);
            result.noun_singular_definite =
                pick_inflection_form(&inflections, &["Eintal", "Bestemt"], &[]);
            result.noun_plural_definite =
                pick_inflection_form(&inflections, &["Fleirtal", "Bestemt"], &[]);
        }
        "adjective" => {
            result.adjective_neuter_form = pick_inflection_form(
                &inflections,
                &["Positiv", "Inkjekjoenn", "Eintal"],
                &["Fleirtal", "Bestemt", "Komparativ", "Superlativ"],
            );
            result.adjective_plural_form = pick_inflection_form(
                &inflections,
                &["Positiv", "Fleirtal"],
                &["Bestemt", "Komparativ", "Superlativ"],
            );
            result.adjective_comparative = pick_inflection_form(
                &inflections,
                &["Komparativ"],
                &["Superlativ", "Bestemt"],
            );
            result.adjective_superlative_indefinite =
                pick_inflection_form(&inflections, &["Superlativ"], &["Bestemt"]);
            result.adjective_superlative_definite =
                pick_inflection_form(&inflections, &["Superlativ", "Bestemt"], &[]);
        }
        "adverb" => {
            result.adverb_comparative =
                pick_inflection_form(&inflections, &["Komparativ"], &["Superlativ"]);
            result.adverb_superlative = pick_inflection_form(&inflections, &["Superlativ"], &[]);
        }
        _ => {}
    }

    Some(result)
}

fn map_ordbok_word_class_to_key(word_class: Option<&str>, hint: &str) -> Option<&'static str> {
    let hinted = hint_to_part_of_speech(hint);
    match word_class.unwrap_or_default() {
        "Verb" => Some("verb"),
        "Substantiv" => Some("noun"),
        "Adjektiv" => Some("adjective"),
        "Adverb" => Some("adverb"),
        "Pronomen" => Some("pronoun"),
        "Talord" => {
            if matches!(hinted, Some(PartOfSpeech::OrdinalNumber)) {
                Some("ordinal_number")
            } else {
                Some("cardinal_number")
            }
        }
        _ => hinted.map(|pos| pos.as_key()),
    }
}

fn collect_article_inflections(article: &OrdbokArticle) -> Vec<OrdbokInflection> {
    article
        .lemmas
        .iter()
        .flat_map(|lemma| lemma.paradigms.iter())
        .flat_map(|paradigm| paradigm.inflections.iter().cloned())
        .collect()
}

fn pick_inflection_form(
    inflections: &[OrdbokInflection],
    required_tags: &[&str],
    forbidden_tags: &[&str],
) -> Option<String> {
    inflections.iter().find_map(|inflection| {
        if !required_tags
            .iter()
            .all(|tag| has_inflection_tag(&inflection.tags, tag))
        {
            return None;
        }
        if forbidden_tags
            .iter()
            .any(|tag| has_inflection_tag(&inflection.tags, tag))
        {
            return None;
        }
        normalize_text_opt(inflection.word_form.clone())
    })
}

fn has_inflection_tag(tags: &[String], tag: &str) -> bool {
    tags.iter().any(|value| value == tag)
}

fn extract_article_definition_texts(article: &OrdbokArticle) -> Vec<String> {
    let mut values = Vec::<String>::new();
    for definition in &article.flat_definitions {
        for content in &definition.content {
            if let Some(text) = normalize_text_opt(content.text_content.clone()) {
                push_unique_text(&mut values, text);
                if values.len() >= 6 {
                    return values;
                }
            }
        }
    }
    values
}

fn merge_or_insert_word_result(results: &mut Vec<GeminiWordResult>, incoming: GeminiWordResult) {
    let incoming_pos = normalize_text_opt(incoming.part_of_speech.clone()).unwrap_or_default();
    let incoming_base = normalize_text_opt(incoming.base_form.clone()).unwrap_or_default();
    if incoming_base.is_empty() {
        return;
    }

    if let Some(existing) = results.iter_mut().find(|candidate| {
        normalize_text_opt(candidate.part_of_speech.clone()).unwrap_or_default() == incoming_pos
            && normalize_text_opt(candidate.base_form.clone()).unwrap_or_default() == incoming_base
    }) {
        merge_word_result(existing, incoming);
    } else {
        results.push(incoming);
    }
}

fn merge_word_result(target: &mut GeminiWordResult, source: GeminiWordResult) {
    for value in source.english {
        push_unique_text(&mut target.english, value);
    }
    for value in source.chinese {
        push_unique_text(&mut target.chinese, value);
    }

    if target.part_of_speech.is_none() {
        target.part_of_speech = source.part_of_speech;
    }
    if target.base_form.is_none() {
        target.base_form = source.base_form;
    }

    if target.verb_present_tense.is_none() {
        target.verb_present_tense = source.verb_present_tense;
    }
    if target.verb_past_tense.is_none() {
        target.verb_past_tense = source.verb_past_tense;
    }
    if target.verb_imperative.is_none() {
        target.verb_imperative = source.verb_imperative;
    }
    if target.noun_plural.is_none() {
        target.noun_plural = source.noun_plural;
    }
    if target.noun_singular_definite.is_none() {
        target.noun_singular_definite = source.noun_singular_definite;
    }
    if target.noun_plural_definite.is_none() {
        target.noun_plural_definite = source.noun_plural_definite;
    }
    if target.adjective_neuter_form.is_none() {
        target.adjective_neuter_form = source.adjective_neuter_form;
    }
    if target.adjective_plural_form.is_none() {
        target.adjective_plural_form = source.adjective_plural_form;
    }
    if target.adjective_comparative.is_none() {
        target.adjective_comparative = source.adjective_comparative;
    }
    if target.adjective_superlative_indefinite.is_none() {
        target.adjective_superlative_indefinite = source.adjective_superlative_indefinite;
    }
    if target.adjective_superlative_definite.is_none() {
        target.adjective_superlative_definite = source.adjective_superlative_definite;
    }
    if target.adverb_comparative.is_none() {
        target.adverb_comparative = source.adverb_comparative;
    }
    if target.adverb_superlative.is_none() {
        target.adverb_superlative = source.adverb_superlative;
    }
}

fn push_unique_text(target: &mut Vec<String>, value: String) {
    let normalized = value.trim();
    if normalized.is_empty() {
        return;
    }
    if !target
        .iter()
        .any(|existing| existing.trim().eq_ignore_ascii_case(normalized))
    {
        target.push(normalized.to_string());
    }
}

fn parse_gemini_word_results(raw_text: &str) -> Result<Vec<GeminiWordResult>, String> {
    let json_text = extract_json_payload(raw_text)
        .ok_or_else(|| "No JSON payload extracted. Please check Gemini response format.".to_string())?;
    let value =
        serde_json::from_str::<Value>(&json_text).map_err(|err| format!("JSON parse failed: {err}"))?;

    if value.is_object() {
        let one = serde_json::from_value::<GeminiWordResult>(value)
            .map_err(|err| format!("Single object parse failed: {err}"))?;
        return Ok(vec![one]);
    }
    if value.is_array() {
        return serde_json::from_value::<Vec<GeminiWordResult>>(value)
            .map_err(|err| format!("Array parse failed: {err}"));
    }
    Err("JSON root must be an object or an array.".to_string())
}

fn extract_json_payload(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return Some(trimmed.to_string());
    }

    let mut candidates = Vec::new();
    if let (Some(start), Some(end)) = (raw.find('['), raw.rfind(']')) {
        if start <= end {
            candidates.push(raw[start..=end].to_string());
        }
    }
    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if start <= end {
            candidates.push(raw[start..=end].to_string());
        }
    }

    candidates
        .into_iter()
        .find(|candidate| serde_json::from_str::<Value>(candidate).is_ok())
}

async fn call_gemini_text(api_key: &str, prompt: &str) -> Result<String, String> {
    let payload_body = serde_json::json!({
        "contents": [
            {
                "parts": [
                    { "text": prompt }
                ]
            }
        ]
    })
    .to_string();
    let mut model_errors = Vec::new();

    for model in GEMINI_MODEL_CANDIDATES {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
        );
        let request = Request::post(&url)
            .header("Content-Type", "application/json")
            .body(payload_body.clone())
            .map_err(|err| format!("Request build failed: {err}"))?;
        let response = request
            .send()
            .await
            .map_err(|err| format!("Request send failed: {err}"))?;

        if !response.ok() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "(Failed to read response body)".to_string());
            if should_try_next_model(status, &body) {
                model_errors.push(format!("{model} -> HTTP {status}"));
                continue;
            }
            return Err(format!("{model} request failed: HTTP {status}: {body}"));
        }

        let parsed = response
            .json::<GeminiResponse>()
            .await
            .map_err(|err| format!("{model} response parse failed: {err}"))?;
        let text = parsed
            .candidates
            .first()
            .and_then(|c| c.content.as_ref())
            .and_then(|content| content.parts.first())
            .and_then(|part| part.text.clone())
            .unwrap_or_default();
        if text.trim().is_empty() {
            return Err(format!("{model} returned empty content."));
        }
        return Ok(text);
    }

    Err(format!(
        "No available Gemini model worked. Check API permission or change model. Attempts: {}",
        model_errors.join(", ")
    ))
}

fn should_try_next_model(status: u16, body: &str) -> bool {
    if status != 404 {
        return false;
    }
    let lower = body.to_lowercase();
    lower.contains("no longer available")
        || lower.contains("not_found")
        || lower.contains("model")
        || lower.contains("not found")
}
