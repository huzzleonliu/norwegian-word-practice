use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::structures::pracresult::{PracticeResult, PracticedWordEntryResult};

const FIXED_PRIVATE_KEY: &str = "NWP_FIXED_PRIVATE_KEY_V1_2026";
const PUBLIC_KEY_BYTES_LEN: usize = 32;
const NONCE_BYTES_LEN: usize = 16;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct EncryptedPracticeResultFile {
    pub payload_version: u8,
    pub username: String,
    pub encryption_key: String,
    pub selected_word_entry_ids: Vec<String>,
    pub practiced_word_entries: String,
    pub nonce: String,
}

pub fn serialize_practice_result_for_export(result: &PracticeResult) -> Result<String, String> {
    let public_key = random_bytes(PUBLIC_KEY_BYTES_LEN);
    let nonce = random_bytes(NONCE_BYTES_LEN);
    let encrypted_entries =
        encrypt_practiced_entries(&result.practiced_word_entries, &public_key, &nonce)?;

    let export_payload = EncryptedPracticeResultFile {
        payload_version: 1,
        username: result.username.clone(),
        encryption_key: STANDARD.encode(&public_key),
        selected_word_entry_ids: result.selected_word_entry_ids.clone(),
        practiced_word_entries: STANDARD.encode(encrypted_entries),
        nonce: STANDARD.encode(&nonce),
    };

    serde_json::to_string_pretty(&export_payload)
        .map_err(|err| format!("练习结果序列化失败：{err}"))
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn parse_practice_result_from_import(content: &str) -> Result<PracticeResult, String> {
    if let Ok(payload) = serde_json::from_str::<EncryptedPracticeResultFile>(content) {
        return decrypt_payload(payload);
    }

    serde_json::from_str::<PracticeResult>(content)
        .map_err(|err| format!("练习结果格式不合法：{err}"))
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn decrypt_payload(payload: EncryptedPracticeResultFile) -> Result<PracticeResult, String> {
    if payload.payload_version > 1 {
        return Err(format!("不支持的练习结果版本：{}", payload.payload_version));
    }

    let public_key = STANDARD
        .decode(payload.encryption_key.trim())
        .map_err(|err| format!("练习结果公钥解析失败：{err}"))?;
    let nonce = STANDARD
        .decode(payload.nonce.trim())
        .map_err(|err| format!("练习结果 nonce 解析失败：{err}"))?;
    let encrypted_entries = STANDARD
        .decode(payload.practiced_word_entries.trim())
        .map_err(|err| format!("练习结果密文解析失败：{err}"))?;

    let practiced_word_entries = decrypt_practiced_entries(&encrypted_entries, &public_key, &nonce)?;

    Ok(PracticeResult {
        username: payload.username,
        encryption_key: payload.encryption_key,
        selected_word_entry_ids: payload.selected_word_entry_ids,
        practiced_word_entries,
    })
}

fn encrypt_practiced_entries(
    entries: &[PracticedWordEntryResult],
    public_key: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let plain =
        serde_json::to_vec(entries).map_err(|err| format!("练习结果明文序列化失败：{err}"))?;
    Ok(apply_stream_cipher(&plain, public_key, nonce))
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn decrypt_practiced_entries(
    cipher: &[u8],
    public_key: &[u8],
    nonce: &[u8],
) -> Result<Vec<PracticedWordEntryResult>, String> {
    let plain = apply_stream_cipher(cipher, public_key, nonce);
    serde_json::from_slice(&plain).map_err(|err| format!("练习结果解密后反序列化失败：{err}"))
}

fn apply_stream_cipher(input: &[u8], public_key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut offset = 0_usize;
    let mut counter = 0_u64;

    while offset < input.len() {
        let mut hasher = Sha256::new();
        hasher.update(FIXED_PRIVATE_KEY.as_bytes());
        hasher.update(public_key);
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
