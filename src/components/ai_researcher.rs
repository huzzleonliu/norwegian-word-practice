use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::Value;

use crate::components::lexicon_browser::serialize_word_bank_csv;
use crate::structures::word_bank_entry::{PartOfSpeech, WordBankEntry};

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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct GeminiWordResult {
    part_of_speech: Option<String>,
    base_form: Option<String>,
    chinese: Vec<String>,
    english: Vec<String>,
    tags: Vec<String>,
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
            tags: normalize_text_list(&value.tags),
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
    single_tags: ReadSignal<String>,
    set_single_tags: WriteSignal<String>,
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
    let (gemini_token, set_gemini_token) = signal(String::new());
    let (form_hint, set_form_hint) = signal("unknown".to_string());
    let (query_word, set_query_word) = signal(String::new());
    let (is_testing, set_is_testing) = signal(false);
    let (is_querying, set_is_querying) = signal(false);

    let test_connectivity = move |_| {
        let token = gemini_token.get_untracked().trim().to_string();
        if token.is_empty() {
            set_status.set("Gemini 连接检测失败：请先输入 API Token。".to_string());
            return;
        }

        set_is_testing.set(true);
        let set_status = set_status;
        let set_is_testing = set_is_testing;
        spawn_local(async move {
            match call_gemini_text(&token, "Reply with exactly: CONNECTED").await {
                Ok(reply) if reply.to_uppercase().contains("CONNECTED") => {
                    set_status.set("Gemini 连通成功。".to_string());
                }
                Ok(reply) => set_status.set(format!("Gemini 有响应，但检测返回异常：{reply}")),
                Err(err) => set_status.set(format!("Gemini 连接检测失败：{err}")),
            }
            set_is_testing.set(false);
        });
    };

    let query_forms = move |_| {
        let token = gemini_token.get_untracked().trim().to_string();
        if token.is_empty() {
            set_status.set("查询失败：请先输入 Gemini API Token。".to_string());
            return;
        }
        let word = query_word.get_untracked().trim().to_string();
        if word.is_empty() {
            set_status.set("查询失败：请输入要查询的词。".to_string());
            return;
        }
        let hint = form_hint.get_untracked();
        set_is_querying.set(true);

        let set_is_querying = set_is_querying;
        let set_status = set_status;
        spawn_local(async move {
            let prompt = build_research_prompt(&word, &hint);
            let result = call_gemini_text(&token, &prompt).await;
            match result {
                Ok(raw_text) => match parse_gemini_word_results(&raw_text) {
                    Ok(parsed_list) => {
                        if parsed_list.is_empty() {
                            set_status.set("查询失败：AI 返回了空数组。".to_string());
                        } else if parsed_list.len() == 1 {
                            let parsed = parsed_list.first().cloned().unwrap_or_default();
                            let pos =
                                normalize_part_of_speech(parsed.part_of_speech.clone(), &hint);
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
                            if let Some(v) = join_pipe(parsed.tags) {
                                set_if_blank(single_tags, set_single_tags, v);
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

                            set_status.set(format!("AI 查询成功：已填充“{word}”的单条结果。"));
                        } else {
                            match build_bulk_csv_from_results(&parsed_list, &hint) {
                                Ok(csv_text) => {
                                    set_bulk_input.set(csv_text);
                                    set_bulk_errors.set(Vec::new());
                                    set_bulk_success_message.set(String::new());
                                    set_status.set(format!(
                                        "AI 查询返回 {} 条结果，已写入“多条添加”输入框。",
                                        parsed_list.len()
                                    ));
                                }
                                Err(err) => {
                                    set_status.set(format!("查询失败：多条结果转 CSV 失败。{err}"));
                                }
                            }
                        }
                    }
                    Err(err) => set_status.set(format!("查询失败：AI 返回无法解析。{err}")),
                },
                Err(err) => set_status.set(format!("查询失败：{err}")),
            }
            set_is_querying.set(false);
        });
    };

    view! {
        <section class="mt-4 rounded-xl border border-slate-800 bg-slate-950/50 p-4">
            <h2 class="mb-3 text-lg font-semibold">"AI 辅助填充（Gemini）"</h2>

            <div class="grid grid-cols-1 gap-3 md:grid-cols-[1fr_auto]">
                <input
                    type="password"
                    placeholder="输入 Gemini API Token"
                    prop:value=move || gemini_token.get()
                    on:input=move |ev| set_gemini_token.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=test_connectivity
                    disabled=move || is_testing.get()
                    class="rounded-lg border border-cyan-700 bg-cyan-700 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-600 disabled:cursor-not-allowed disabled:opacity-60"
                >
                    {move || if is_testing.get() { "检测中..." } else { "检测连通" }}
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
                        .map(|(value, label)| view! { <option value=*value>{*label}</option> })
                        .collect_view()}
                </select>
                <input
                    type="text"
                    placeholder="输入单词"
                    prop:value=move || query_word.get()
                    on:input=move |ev| set_query_word.set(event_target_value(&ev))
                    class="rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm"
                />
                <button
                    type="button"
                    on:click=query_forms
                    disabled=move || is_querying.get()
                    class="rounded-lg border border-emerald-700 bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 disabled:cursor-not-allowed disabled:opacity-60"
                >
                    {move || if is_querying.get() { "查询中..." } else { "查询并分流" }}
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
  \"tags\": [],\n\
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

fn parse_gemini_word_results(raw_text: &str) -> Result<Vec<GeminiWordResult>, String> {
    let json_text = extract_json_payload(raw_text)
        .ok_or_else(|| "未提取到 JSON 内容。请检查 Gemini 返回格式。".to_string())?;
    let value =
        serde_json::from_str::<Value>(&json_text).map_err(|err| format!("JSON 解析失败：{err}"))?;

    if value.is_object() {
        let one = serde_json::from_value::<GeminiWordResult>(value)
            .map_err(|err| format!("单条对象解析失败：{err}"))?;
        return Ok(vec![one]);
    }
    if value.is_array() {
        return serde_json::from_value::<Vec<GeminiWordResult>>(value)
            .map_err(|err| format!("数组解析失败：{err}"));
    }
    Err("JSON 根节点必须是对象或数组。".to_string())
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
            .map_err(|err| format!("请求构建失败：{err}"))?;
        let response = request
            .send()
            .await
            .map_err(|err| format!("请求发送失败：{err}"))?;

        if !response.ok() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "（无法读取响应体）".to_string());
            if should_try_next_model(status, &body) {
                model_errors.push(format!("{model} -> HTTP {status}"));
                continue;
            }
            return Err(format!("{model} 请求失败：HTTP {status}：{body}"));
        }

        let parsed = response
            .json::<GeminiResponse>()
            .await
            .map_err(|err| format!("{model} 响应解析失败：{err}"))?;
        let text = parsed
            .candidates
            .first()
            .and_then(|c| c.content.as_ref())
            .and_then(|content| content.parts.first())
            .and_then(|part| part.text.clone())
            .unwrap_or_default();
        if text.trim().is_empty() {
            return Err(format!("{model} 返回为空。"));
        }
        return Ok(text);
    }

    Err(format!(
        "可用模型均不可用，请检查 Gemini API 权限或更换模型。尝试记录：{}",
        model_errors.join("，")
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
