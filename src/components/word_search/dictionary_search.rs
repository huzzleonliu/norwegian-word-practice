use gloo_net::http::Request;
use serde::Deserialize;

use crate::structures::word_bank_entry::PartOfSpeech;

use super::{
    GeminiWordResult, hint_to_part_of_speech, merge_or_insert_word_result, normalize_part_of_speech,
    normalize_text_opt,
};

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
      }
    }
  }
}
"#;
const GOOGLE_TRANSLATE_ENDPOINT: &str = "https://translation.googleapis.com/language/translate/v2";

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
        return Err(format!("Google Translate request failed: HTTP {status}: {body}"));
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
    if filtered.is_empty() {
        merged
    } else {
        filtered
    }
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
