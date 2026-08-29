//! Ordbok 与 Google Translate 适配层：负责请求、解析和词形映射。

use gloo_net::http::Request;
use serde::Deserialize;

use crate::structures::word_bank_entry::PartOfSpeech;

use super::{
    GeminiWordResult, hint_to_part_of_speech, merge_or_insert_word_result,
    normalize_part_of_speech, normalize_text_opt,
};

const ORDBOK_GRAPHQL_ENDPOINT: &str = "https://api.ordbokapi.org/graphql";
/// 第一步：只取条目 id / 词典 / 词性 / 原型，避免嵌套 paradigms 触发 QUERY_TOO_COMPLEX。
const ORDBOK_SUGGESTIONS_QUERY: &str = r#"
query LookUp($word: String!) {
  suggestions(word: $word) {
    exact {
      word
      articles {
        id
        dictionary
        wordClass
        lemmas {
          lemma
        }
      }
    }
  }
}
"#;
/// 第二步：按 article id + dictionary 拉取词形变化。
const ORDBOK_ARTICLE_QUERY: &str = r#"
query ArticleForms($id: Int!, $dictionary: Dictionary!) {
  article(id: $id, dictionary: $dictionary) {
    id
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
  }
}
"#;
const GOOGLE_TRANSLATE_ENDPOINT: &str = "https://translation.googleapis.com/language/translate/v2";

#[derive(Debug, Deserialize)]
struct OrdbokGraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<OrdbokGraphQlError>,
}

#[derive(Debug, Deserialize)]
struct OrdbokGraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct OrdbokSuggestionsData {
    suggestions: OrdbokSuggestions,
}

#[derive(Debug, Deserialize)]
struct OrdbokArticleData {
    article: Option<OrdbokArticle>,
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
    articles: Vec<OrdbokArticleRef>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrdbokArticleRef {
    id: i64,
    dictionary: String,
    word_class: Option<String>,
    #[serde(default)]
    lemmas: Vec<OrdbokLemmaLite>,
}

#[derive(Debug, Deserialize)]
struct OrdbokLemmaLite {
    lemma: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrdbokArticle {
    #[serde(default)]
    word_class: Option<String>,
    #[serde(default)]
    lemmas: Vec<OrdbokLemma>,
}

#[derive(Clone, Debug, Deserialize)]
struct OrdbokLemma {
    lemma: String,
    #[serde(default)]
    paradigms: Vec<OrdbokParadigm>,
}

#[derive(Clone, Debug, Deserialize)]
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
struct GoogleTranslateResponse {
    data: Option<GoogleTranslateData>,
    error: Option<GoogleTranslateError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateData {
    #[serde(default)]
    translations: Vec<GoogleTranslateItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslateItem {
    translated_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateError {
    message: Option<String>,
}

pub(crate) async fn query_word_with_ordbok(
    word: &str,
    hint: &str,
) -> Result<Vec<GeminiWordResult>, String> {
    // Ordbok 当前对「suggestions + paradigms」单次查询会报 QUERY_TOO_COMPLEX，
    // 因此拆成：suggestions 定位条目 → article(id, dictionary) 拉词形。
    let suggestions = ordbok_graphql::<OrdbokSuggestionsData>(
        ORDBOK_SUGGESTIONS_QUERY,
        serde_json::json!({ "word": word }),
    )
    .await?;

    let Some(data) = suggestions else {
        return Ok(Vec::new());
    };

    let mut article_refs = data
        .suggestions
        .exact
        .into_iter()
        .flat_map(|exact| {
            let exact_word = exact.word;
            exact
                .articles
                .into_iter()
                .map(move |article| (exact_word.clone(), article))
        })
        .collect::<Vec<_>>();

    // 优先 Bokmål；若有 Bokmål 条目则只拉 Bokmål，避免多词典重复请求。
    let has_bokmal = article_refs
        .iter()
        .any(|(_, article)| article.dictionary == "Bokmaalsordboka");
    if has_bokmal {
        article_refs.retain(|(_, article)| article.dictionary == "Bokmaalsordboka");
    } else {
        article_refs.sort_by_key(|(_, article)| article.dictionary.clone());
    }
    let mut merged = Vec::<GeminiWordResult>::new();
    let mut seen_ids = std::collections::HashSet::<(i64, String)>::new();
    for (exact_word, article_ref) in article_refs {
        let key = (article_ref.id, article_ref.dictionary.clone());
        if !seen_ids.insert(key) {
            continue;
        }

        let article_data = ordbok_graphql::<OrdbokArticleData>(
            ORDBOK_ARTICLE_QUERY,
            serde_json::json!({
                "id": article_ref.id,
                "dictionary": article_ref.dictionary,
            }),
        )
        .await?;

        let fallback_lemmas = article_ref
            .lemmas
            .iter()
            .map(|lemma| OrdbokLemma {
                lemma: lemma.lemma.clone(),
                paradigms: Vec::new(),
            })
            .collect::<Vec<_>>();

        let mut article = match article_data.and_then(|data| data.article) {
            Some(article) => article,
            None => OrdbokArticle {
                word_class: article_ref.word_class.clone(),
                lemmas: fallback_lemmas.clone(),
            },
        };
        if article.word_class.is_none() {
            article.word_class = article_ref.word_class.clone();
        }
        if article.lemmas.is_empty() {
            article.lemmas = fallback_lemmas;
        }

        if let Some(item) = convert_ordbok_article(&exact_word, &article, hint) {
            merge_or_insert_word_result(&mut merged, item);
        }
    }

    Ok(filter_results_by_hint(merged, hint))
}

async fn ordbok_graphql<T>(query: &str, variables: serde_json::Value) -> Result<Option<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let payload_body = serde_json::json!({
        "query": query,
        "variables": variables,
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
        .json::<OrdbokGraphQlResponse<T>>()
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
    Ok(parsed.data)
}

fn filter_results_by_hint(merged: Vec<GeminiWordResult>, hint: &str) -> Vec<GeminiWordResult> {
    let Some(expected_pos) = hint_to_part_of_speech(hint).map(|pos| pos.as_key().to_string())
    else {
        return merged;
    };
    let filtered = merged
        .iter()
        .filter(|item| normalize_part_of_speech(item.part_of_speech.clone(), hint) == expected_pos)
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        merged
    } else {
        filtered
    }
}

/// 使用 Google Translate 回填中英文释义（不改词形字段）。
/// 源语言固定为挪威语（`no`），避免短词被误判成英语等其它语言。
pub(crate) async fn enrich_results_with_google_translate(
    results: &mut [GeminiWordResult],
    api_key: &str,
) -> Result<(), String> {
    for item in results.iter_mut() {
        let Some(base_form) = normalize_text_opt(item.base_form.clone()) else {
            continue;
        };
        let english = translate_text_with_google(api_key, &base_form, "en").await?;
        let chinese = translate_text_with_google(api_key, &base_form, "zh-CN").await?;
        item.english = vec![english];
        item.chinese = vec![chinese];
    }
    Ok(())
}

/// 用简短词做连通性探测，返回示例翻译文本。
pub(crate) async fn test_google_translate_connectivity(api_key: &str) -> Result<String, String> {
    translate_text_with_google(api_key, "hei", "en").await
}

async fn translate_text_with_google(
    api_key: &str,
    text: &str,
    target_language: &str,
) -> Result<String, String> {
    let payload_body = serde_json::json!({
        "q": text,
        "source": "no",
        "target": target_language,
        "format": "text",
    })
    .to_string();
    let request_url = format!("{GOOGLE_TRANSLATE_ENDPOINT}?key={api_key}");
    let request = Request::post(&request_url)
        .header("Content-Type", "application/json")
        .body(payload_body)
        .map_err(|err| format!("Google Translate request build failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("Google Translate request failed: {err}"))?;

    if !response.ok() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(failed to read response body)".to_string());
        return Err(format!(
            "Google Translate request failed: HTTP {status}: {body}"
        ));
    }

    let parsed = response
        .json::<GoogleTranslateResponse>()
        .await
        .map_err(|err| format!("Google Translate response parse failed: {err}"))?;
    if let Some(error) = parsed.error {
        let message = error
            .message
            .unwrap_or_else(|| "unknown Google Translate error".to_string());
        return Err(message);
    }
    let translated = parsed
        .data
        .and_then(|data| data.translations.into_iter().next())
        .and_then(|item| item.translated_text)
        .and_then(|text| normalize_text_opt(Some(decode_html_entities(&text))))
        .ok_or_else(|| "Google Translate returned empty translation.".to_string())?;
    Ok(translated)
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
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
        english: Vec::new(),
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
    };

    match normalized_pos.as_str() {
        "verb" => {
            result.verb_present_tense =
                pick_inflection_form(&inflections, &["Presens"], &["Passiv", "SPassiv"]);
            result.verb_past_tense =
                pick_inflection_form(&inflections, &["Preteritum"], &["Passiv", "SPassiv"]);
            result.verb_imperative =
                pick_inflection_form(&inflections, &["Imperativ"], &["Passiv", "SPassiv"]);
            result.verb_present_participle =
                pick_inflection_form(&inflections, &["PresensPartisipp"], &[]);
            result.verb_past_participle =
                pick_inflection_form(&inflections, &["PerfektPartisipp"], &["Passiv", "SPassiv"]);
            result.verb_passive_infinitive =
                pick_inflection_form(&inflections, &["Infinitiv", "Passiv"], &[])
                    .or_else(|| pick_inflection_form(&inflections, &["Infinitiv", "SPassiv"], &[]));
            result.verb_passive_present =
                pick_inflection_form(&inflections, &["Presens", "Passiv"], &[])
                    .or_else(|| pick_inflection_form(&inflections, &["Presens", "SPassiv"], &[]));
            result.verb_passive_past =
                pick_inflection_form(&inflections, &["Preteritum", "Passiv"], &[]).or_else(|| {
                    pick_inflection_form(&inflections, &["Preteritum", "SPassiv"], &[])
                });
        }
        "noun" => {
            result.noun_plural = pick_inflection_form(&inflections, &["Fleirtal"], &["Bestemt"]);
            result.noun_singular_definite =
                pick_inflection_form(&inflections, &["Eintal", "Bestemt"], &[]);
            result.noun_plural_definite =
                pick_inflection_form(&inflections, &["Fleirtal", "Bestemt"], &[]);
            result.noun_singular_definite_genitive =
                append_genitive(result.noun_singular_definite.as_deref());
            result.noun_plural_definite_genitive =
                append_genitive(result.noun_plural_definite.as_deref());
            result.noun_singular_indefinite_genitive = append_genitive(result.base_form.as_deref());
            result.noun_plural_indefinite_genitive = append_genitive(result.noun_plural.as_deref());
        }
        "adjective" => {
            result.adjective_feminine_form = pick_inflection_form(
                &inflections,
                &["Positiv", "Hokjoenn", "Eintal"],
                &["Fleirtal", "Bestemt", "Komparativ", "Superlativ"],
            )
            .or_else(|| {
                pick_inflection_form(
                    &inflections,
                    &["Positiv", "HankjoennHokjoenn", "Eintal"],
                    &["Fleirtal", "Bestemt", "Komparativ", "Superlativ"],
                )
            });
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
            result.adjective_comparative =
                pick_inflection_form(&inflections, &["Komparativ"], &["Superlativ", "Bestemt"]);
            result.adjective_superlative_indefinite = pick_inflection_form(
                &inflections,
                &["Superlativ", "Ubestemt"],
                &["Bestemt"],
            )
            .or_else(|| pick_inflection_form(&inflections, &["Superlativ"], &["Bestemt"]));
            result.adjective_superlative_definite =
                pick_inflection_form(&inflections, &["Superlativ", "Bestemt"], &[]);
        }
        "pronoun" => {
            result.pronoun_object =
                pick_inflection_form(&inflections, &["Objekt"], &["Fleirtal", "Refleksiv"])
                    .or_else(|| pick_inflection_form(&inflections, &["Akkusativ"], &["Fleirtal"]));
            result.pronoun_reflexive =
                pick_inflection_form(&inflections, &["Refleksiv"], &["Fleirtal"]);
            result.pronoun_plural_subject = pick_inflection_form(
                &inflections,
                &["Fleirtal", "Subjekt"],
                &["Objekt", "Refleksiv"],
            );
            result.pronoun_plural_object =
                pick_inflection_form(&inflections, &["Fleirtal", "Objekt"], &["Refleksiv"])
                    .or_else(|| {
                        pick_inflection_form(&inflections, &["Fleirtal", "Akkusativ"], &[])
                    });
            result.pronoun_plural_reflexive =
                pick_inflection_form(&inflections, &["Fleirtal", "Refleksiv"], &[]);
        }
        "determinative" => {
            result.determinative_feminine_form = pick_inflection_form(
                &inflections,
                &["Hokjoenn", "Eintal"],
                &["Inkjekjoenn", "Fleirtal"],
            )
            .or_else(|| {
                pick_inflection_form(
                    &inflections,
                    &["Hankjoenn", "Eintal"],
                    &["Inkjekjoenn", "Fleirtal"],
                )
            })
            .or_else(|| {
                pick_inflection_form(
                    &inflections,
                    &["HankjoennHokjoenn", "Eintal"],
                    &["Inkjekjoenn", "Fleirtal"],
                )
            });
            result.determinative_neuter_form =
                pick_inflection_form(&inflections, &["Inkjekjoenn", "Eintal"], &["Fleirtal"]);
            result.determinative_plural_form =
                pick_inflection_form(&inflections, &["Fleirtal"], &[]);
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
        "Determinativ" | "Artikkel" => Some("determinative"),
        "Adverb" => Some("adverb"),
        "Pronomen" => {
            if matches!(hinted, Some(PartOfSpeech::Determinative)) {
                Some("determinative")
            } else {
                Some("pronoun")
            }
        }
        "Preposisjon" => Some("preposition"),
        "Konjunksjon" => Some("conjunction"),
        "Subjunksjon" => Some("subjunction"),
        "Interjeksjon" => Some("interjection"),
        "Frase" | "Uttrykk" | "Flerordsuttrykk" => Some("phrase"),
        "Talord" => {
            if matches!(hinted, Some(PartOfSpeech::Adjective)) {
                Some("adjective")
            } else {
                Some("determinative")
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

fn append_genitive(form: Option<&str>) -> Option<String> {
    let form = form?.trim();
    if form.is_empty() {
        None
    } else {
        Some(format!("{form}s"))
    }
}
