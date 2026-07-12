use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::Value;

use super::GeminiWordResult;

const GEMINI_MODEL_CANDIDATES: [&str; 3] =
    ["gemini-2.5-flash", "gemini-2.5-pro", "gemini-1.5-flash"];

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

pub(crate) async fn test_gemini_connectivity(api_key: &str) -> Result<(), String> {
    let reply = call_gemini_text(api_key, "Reply with exactly: CONNECTED").await?;
    if reply.to_uppercase().contains("CONNECTED") {
        Ok(())
    } else {
        Err(format!("Unexpected health-check reply: {reply}"))
    }
}

pub(crate) async fn query_word_with_gemini(
    token: &str,
    word: &str,
    hint: &str,
) -> Result<Vec<GeminiWordResult>, String> {
    let prompt = build_research_prompt(word, hint);
    let raw_text = call_gemini_text(token, &prompt).await?;
    parse_gemini_word_results(&raw_text)
        .map_err(|err| format!("Gemini response parse failed: {err}"))
}

fn build_research_prompt(word: &str, hint: &str) -> String {
    format!(
        "你是挪威语词形助手。已知用户输入单词是：{word}；它的已知形式提示是：{hint}。\n\
请输出该词在词库中的完整信息，并且只返回 JSON（不要 markdown，不要解释）。\n\
如果存在多个合理词条（例如同形异义、不同词性），请返回 JSON 数组。\n\
如果只有一个词条，可返回单个 JSON 对象。\n\
对象结构严格如下（字段可为 null 或空数组）：\n\
{{\n\
  \"part_of_speech\": \"noun|verb|adjective|pronoun|determinative|adverb|preposition|conjunction|subjunction|interjection\",\n\
  \"base_form\": \"\",\n\
  \"chinese\": [],\n\
  \"english\": [],\n\
  \"verb_present_tense\": null,\n\
  \"verb_past_tense\": null,\n\
  \"verb_imperative\": null,\n\
  \"verb_present_participle\": null,\n\
  \"verb_past_participle\": null,\n\
  \"verb_passive_infinitive\": null,\n\
  \"verb_passive_present\": null,\n\
  \"verb_passive_past\": null,\n\
  \"noun_plural\": null,\n\
  \"noun_singular_definite\": null,\n\
  \"noun_plural_definite\": null,\n\
  \"noun_singular_definite_genitive\": null,\n\
  \"noun_plural_definite_genitive\": null,\n\
  \"noun_singular_indefinite_genitive\": null,\n\
  \"noun_plural_indefinite_genitive\": null,\n\
  \"adjective_feminine_form\": null,\n\
  \"adjective_neuter_form\": null,\n\
  \"adjective_plural_form\": null,\n\
  \"adjective_comparative\": null,\n\
  \"adjective_superlative_indefinite\": null,\n\
  \"adjective_superlative_definite\": null,\n\
  \"pronoun_object\": null,\n\
  \"pronoun_reflexive\": null,\n\
  \"pronoun_plural_subject\": null,\n\
  \"pronoun_plural_object\": null,\n\
  \"pronoun_plural_reflexive\": null,\n\
  \"determinative_feminine_form\": null,\n\
  \"determinative_neuter_form\": null,\n\
  \"determinative_plural_form\": null,\n\
  \"adverb_comparative\": null,\n\
  \"adverb_superlative\": null\n\
}}"
    )
}

fn parse_gemini_word_results(raw_text: &str) -> Result<Vec<GeminiWordResult>, String> {
    let json_text = extract_json_payload(raw_text).ok_or_else(|| {
        "No JSON payload extracted. Please check Gemini response format.".to_string()
    })?;
    let value = serde_json::from_str::<Value>(&json_text)
        .map_err(|err| format!("JSON parse failed: {err}"))?;

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
