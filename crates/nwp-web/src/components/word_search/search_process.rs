//! 词典查询流程：Ordbok → Google Translate → Gemini，必要时按原型二次查询。

use crate::structures::word_bank_entry::UiLanguage;
use crate::utils::i18n::tr;

use super::dictionary_search::{
    enrich_results_with_google_translate, query_word_with_ordbok,
};
use super::gemini_search::query_word_with_gemini;
use super::{GeminiWordResult, merge_word_result, normalize_part_of_speech, normalize_text_opt};

/// 一次查询所需的输入参数。
pub struct SearchRequest {
    pub language: UiLanguage,
    pub word: String,
    pub hint: String,
    pub gemini_token: String,
    pub google_translate_token: String,
    pub google_translate_verified: bool,
    pub ordbok_only: bool,
}

/// 查询成功后的结果。
pub struct SearchSuccess {
    pub results: Vec<GeminiWordResult>,
    pub source_label: String,
}

/// 执行完整查询流程：
/// 1) Ordbok（可选 Google Translate / Gemini 补全）
/// 2) 未命中则回退 Gemini
/// 3) 若查询词与结果原型不一致，用原型再查一轮并以二次结果为准
pub async fn run_word_search(
    request: SearchRequest,
    mut report: impl FnMut(String),
) -> Result<SearchSuccess, String> {
    let language = request.language;
    report(format!(
        "{} \"{}\"",
        tr(language, "开始查询：", "Start query:"),
        request.word
    ));

    let first = run_search_round(&request, &request.word, &mut report).await?;
    report(format!(
        "{} {} {}（{}）",
        tr(language, "首轮查询完成：", "First-round query done:"),
        first.results.len(),
        tr(language, "条结果", "result(s)"),
        first.source_label
    ));

    let Some(base_form) = primary_base_form(&first.results) else {
        return Ok(first);
    };

    if words_match(&request.word, &base_form) {
        report(tr(
            language,
            "查询词与结果原型一致，采用本轮结果。",
            "Query word matches base form; using this round.",
        )
        .to_string());
        return Ok(first);
    }

    report(format!(
        "{} \"{}\" {} \"{}\"，{}",
        tr(language, "查询词", "Query word"),
        request.word,
        tr(language, "与原型", "differs from base form"),
        base_form,
        tr(
            language,
            "将用原型再次查询并以二次结果为准。",
            "re-querying with base form; second round will be authoritative.",
        )
    ));

    match run_search_round(&request, &base_form, &mut report).await {
        Ok(second) => {
            report(format!(
                "{} \"{}\"：{} {}（{}）",
                tr(
                    language,
                    "原型二次查询完成，采用此结果",
                    "Base-form re-query done; using this result",
                ),
                base_form,
                second.results.len(),
                tr(language, "条", "entries"),
                second.source_label
            ));
            Ok(SearchSuccess {
                results: second.results,
                source_label: format!(
                    "{} · {}",
                    second.source_label,
                    tr(language, "原型二次查询", "base-form re-query")
                ),
            })
        }
        Err(err) => {
            report(format!(
                "{} {err}；{}",
                tr(
                    language,
                    "原型二次查询失败：",
                    "Base-form re-query failed:",
                ),
                tr(
                    language,
                    "保留首轮结果。",
                    "keeping first-round results.",
                )
            ));
            Ok(first)
        }
    }
}

async fn run_search_round(
    request: &SearchRequest,
    word: &str,
    report: &mut impl FnMut(String),
) -> Result<SearchSuccess, String> {
    let language = request.language;
    let hint = request.hint.as_str();
    let gemini_token = request.gemini_token.as_str();
    let google_translate_token = request.google_translate_token.as_str();
    let google_translate_verified = request.google_translate_verified;
    let ordbok_only = request.ordbok_only;

    report(format!(
        "{} \"{}\" → Ordbok",
        tr(language, "正在检索", "Looking up"),
        word
    ));

    match query_word_with_ordbok(word, hint).await {
        Ok(mut results) if !results.is_empty() => {
            let mut source_label = tr(language, "Ordbok API", "Ordbok API").to_string();
            let can_use_google_translate =
                !google_translate_token.is_empty() && google_translate_verified;
            let mut used_google_translate = false;

            if can_use_google_translate {
                report(tr(
                    language,
                    "Ordbok 已命中，尝试 Google Translate 回填中英。",
                    "Ordbok hit; trying Google Translate for Chinese/English.",
                )
                .to_string());
                match enrich_results_with_google_translate(&mut results, google_translate_token)
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
                        report(tr(
                            language,
                            "Google Translate 回填成功。",
                            "Google Translate fill succeeded.",
                        )
                        .to_string());
                    }
                    Err(err) => {
                        report(format!(
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
                report(tr(
                    language,
                    "调用 Gemini 补全缺失字段。",
                    "Calling Gemini to fill missing fields.",
                )
                .to_string());
                match query_word_with_gemini(gemini_token, word, hint).await {
                    Ok(gemini_results) => {
                        fill_missing_fields_from_gemini(&mut results, &gemini_results, hint);
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
                        report(tr(
                            language,
                            "Gemini 补全完成。",
                            "Gemini fill completed.",
                        )
                        .to_string());
                    }
                    Err(err) => {
                        report(format!(
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

            Ok(SearchSuccess {
                results,
                source_label,
            })
        }
        Ok(_) => {
            report(tr(
                language,
                "Ordbok 未命中。",
                "No Ordbok match.",
            )
            .to_string());
            if ordbok_only {
                return Err(tr(
                    language,
                    "Ordbok API 未命中，且已开启“仅 Ordbok”模式，不回退 Gemini。",
                    "No Ordbok match and \"Ordbok-only\" mode is enabled; Gemini fallback skipped.",
                )
                .to_string());
            }
            if gemini_token.is_empty() {
                return Err(tr(
                    language,
                    "Ordbok API 未命中，且未提供 Gemini Token，无法回退 AI 查询。",
                    "No Ordbok match and Gemini token is missing; cannot fallback to AI.",
                )
                .to_string());
            }
            report(tr(
                language,
                "回退到 Gemini 查询。",
                "Falling back to Gemini query.",
            )
            .to_string());
            let results = query_word_with_gemini(gemini_token, word, hint)
                .await
                .map_err(|err| {
                    format!(
                        "{} {err}",
                        tr(
                            language,
                            "查询失败：Ordbok 未命中，Gemini 回退也失败。",
                            "Query failed: no Ordbok match and Gemini fallback also failed.",
                        )
                    )
                })?;
            Ok(SearchSuccess {
                results,
                source_label: tr(
                    language,
                    "Gemini（Ordbok 未命中后回退）",
                    "Gemini (fallback after no Ordbok match)",
                )
                .to_string(),
            })
        }
        Err(ordbok_err) => {
            report(format!(
                "{} {ordbok_err}",
                tr(language, "Ordbok 请求失败：", "Ordbok request failed:")
            ));
            if ordbok_only {
                return Err(format!(
                    "{} {ordbok_err}",
                    tr(
                        language,
                        "查询失败：Ordbok API 请求失败，且已开启“仅 Ordbok”模式。",
                        "Query failed: Ordbok API request failed and \"Ordbok-only\" mode is enabled.",
                    )
                ));
            }
            if gemini_token.is_empty() {
                return Err(format!(
                    "{} {ordbok_err}",
                    tr(
                        language,
                        "查询失败：Ordbok API 请求失败，且未提供 Gemini Token。",
                        "Query failed: Ordbok API request failed and Gemini token is missing.",
                    )
                ));
            }
            report(tr(
                language,
                "回退到 Gemini 查询。",
                "Falling back to Gemini query.",
            )
            .to_string());
            let results = query_word_with_gemini(gemini_token, word, hint)
                .await
                .map_err(|ai_err| {
                    format!(
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
                    )
                })?;
            Ok(SearchSuccess {
                results,
                source_label: format!(
                    "{} ({ordbok_err})",
                    tr(
                        language,
                        "Gemini（Ordbok 失败后回退）",
                        "Gemini (fallback after Ordbok error)",
                    )
                ),
            })
        }
    }
}

fn primary_base_form(results: &[GeminiWordResult]) -> Option<String> {
    results
        .iter()
        .find_map(|item| normalize_text_opt(item.base_form.clone()))
}

fn words_match(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
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
