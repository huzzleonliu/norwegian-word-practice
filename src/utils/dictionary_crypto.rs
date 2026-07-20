//! 词库加密工具：
//! - localStorage 持久化密文（核心）
//! - 导入/导出加密格式（`.nwpdict`）
//! - 兼容旧版明文 localStorage 与明文 CSV 导入

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::structures::word_bank_entry::WordBankEntry;

const FIXED_PRIVATE_KEY: &str = "NWP_DICT_FIXED_PRIVATE_KEY_V1_2026";
const NONCE_BYTES_LEN: usize = 16;
const PAYLOAD_VERSION: u8 = 1;
const PURPOSE_STORAGE: &str = "lexicon_storage_v1";
const PURPOSE_EXPORT: &str = "lexicon_export_v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlainLexiconPayload {
    source_name: String,
    entries: Vec<WordBankEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct EncryptedEnvelope {
    payload_version: u8,
    purpose: String,
    nonce: String,
    cipher_text: String,
}

/// 将词库序列化为 localStorage 密文字符串。
pub fn encrypt_lexicon_storage_payload(
    entries: &[WordBankEntry],
    source_name: &str,
) -> Result<String, String> {
    let plain_payload = PlainLexiconPayload {
        source_name: source_name.to_string(),
        entries: entries.to_vec(),
    };
    let plain = serde_json::to_vec(&plain_payload)
        .map_err(|err| format!("词库存储序列化失败：{err}"))?;
    encrypt_envelope(&plain, PURPOSE_STORAGE)
}

/// 从 localStorage 字符串恢复词库。
///
/// 兼容路径：
/// 1) 新版密文 envelope（推荐）；
/// 2) 旧版明文 JSON（迁移兼容）。
pub fn decrypt_lexicon_storage_payload(raw: &str) -> Result<(Vec<WordBankEntry>, String), String> {
    // 先兼容旧版明文存储。
    if let Ok(plain_payload) = serde_json::from_str::<PlainLexiconPayload>(raw) {
        return Ok((plain_payload.entries, plain_payload.source_name));
    }

    let plain = decrypt_envelope(raw, PURPOSE_STORAGE)?;
    let plain_payload: PlainLexiconPayload = serde_json::from_slice(&plain)
        .map_err(|err| format!("词库存储解密后格式无效：{err}"))?;
    Ok((plain_payload.entries, plain_payload.source_name))
}

/// 将 CSV 内容封装为加密导出格式（`.nwpdict`）。
pub fn serialize_encrypted_lexicon_export(csv_content: &str) -> Result<String, String> {
    encrypt_envelope(csv_content.as_bytes(), PURPOSE_EXPORT)
}

/// 解析导入内容：
/// - 若是 `.nwpdict` 加密格式则先解密；
/// - 否则按明文 CSV 原样返回（兼容旧流程）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn parse_lexicon_import_content(content: &str) -> Result<String, String> {
    if let Some(envelope) = parse_encrypted_envelope(content)? {
        if envelope.purpose != PURPOSE_EXPORT {
            return Err("词库导入失败：不支持的加密用途。".to_string());
        }
        let plain = decrypt_envelope_with_parsed(&envelope)?;
        return String::from_utf8(plain)
            .map_err(|err| format!("词库导入失败：解密后文本编码无效：{err}"));
    }
    Ok(content.to_string())
}

fn encrypt_envelope(plain: &[u8], purpose: &str) -> Result<String, String> {
    let nonce = random_bytes(NONCE_BYTES_LEN);
    let cipher = apply_stream_cipher(plain, purpose.as_bytes(), &nonce);
    let envelope = EncryptedEnvelope {
        payload_version: PAYLOAD_VERSION,
        purpose: purpose.to_string(),
        nonce: STANDARD.encode(&nonce),
        cipher_text: STANDARD.encode(cipher),
    };
    serde_json::to_string_pretty(&envelope).map_err(|err| format!("词库加密封装失败：{err}"))
}

fn decrypt_envelope(raw: &str, expected_purpose: &str) -> Result<Vec<u8>, String> {
    let envelope = serde_json::from_str::<EncryptedEnvelope>(raw)
        .map_err(|err| format!("词库密文格式无效：{err}"))?;
    if envelope.purpose != expected_purpose {
        return Err("词库密文用途不匹配。".to_string());
    }
    decrypt_envelope_with_parsed(&envelope)
}

fn decrypt_envelope_with_parsed(envelope: &EncryptedEnvelope) -> Result<Vec<u8>, String> {
    if envelope.payload_version > PAYLOAD_VERSION {
        return Err(format!(
            "词库密文版本过新：{}（当前支持 <= {}）",
            envelope.payload_version, PAYLOAD_VERSION
        ));
    }
    let nonce = STANDARD
        .decode(envelope.nonce.trim())
        .map_err(|err| format!("词库 nonce 解析失败：{err}"))?;
    let cipher = STANDARD
        .decode(envelope.cipher_text.trim())
        .map_err(|err| format!("词库密文解析失败：{err}"))?;
    Ok(apply_stream_cipher(
        &cipher,
        envelope.purpose.as_bytes(),
        &nonce,
    ))
}

/// 尝试从内容中读取加密 envelope。
/// 返回 None 表示按明文 CSV 处理。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn parse_encrypted_envelope(content: &str) -> Result<Option<EncryptedEnvelope>, String> {
    let trimmed = content.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return Ok(None);
    }

    match serde_json::from_str::<EncryptedEnvelope>(trimmed) {
        Ok(envelope) => {
            // `purpose` 为空时视为普通 JSON 文本，回退明文路径。
            if envelope.purpose.trim().is_empty() || envelope.cipher_text.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(envelope))
            }
        }
        Err(_) => Ok(None),
    }
}

fn apply_stream_cipher(input: &[u8], purpose: &[u8], nonce: &[u8]) -> Vec<u8> {
    // 流式异或：基于 (固定私钥 + purpose + nonce + counter) 逐块生成 keystream。
    let mut out = Vec::with_capacity(input.len());
    let mut offset = 0_usize;
    let mut counter = 0_u64;

    while offset < input.len() {
        let mut hasher = Sha256::new();
        hasher.update(FIXED_PRIVATE_KEY.as_bytes());
        hasher.update(purpose);
        hasher.update(nonce);
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        for byte in block {
            if offset >= input.len() {
                break;
            }
            out.push(input[offset] ^ byte);
            offset += 1;
        }
        counter = counter.wrapping_add(1);
    }

    out
}

fn random_bytes(len: usize) -> Vec<u8> {
    #[cfg(target_arch = "wasm32")]
    {
        (0..len)
            .map(|_| (js_sys::Math::random() * 256.0).floor() as u8)
            .collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            out.push((seed >> 32) as u8);
        }
        out
    }
}
