//! 从 Solana Candy Machine 读取已售数量。

use base64::Engine;
use gloo_net::http::Request;
use serde_json::Value;

use crate::structures::nft_collection::{
    CANDY_MACHINE_ID, ITEMS_REDEEMED_OFFSET, SOLANA_RPC_URL,
};

/// 查询 Candy Machine 的 `items_redeemed`（已售数量）。
pub async fn fetch_items_redeemed() -> Result<u64, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            CANDY_MACHINE_ID,
            { "encoding": "base64", "commitment": "confirmed" }
        ]
    });
    let response = Request::post(SOLANA_RPC_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|err| format!("构造请求失败: {err}"))?
        .send()
        .await
        .map_err(|err| format!("RPC 请求失败: {err}"))?;
    let payload: Value = response
        .json()
        .await
        .map_err(|err| format!("解析 RPC 响应失败: {err}"))?;
    if let Some(err) = payload.get("error") {
        return Err(format!("RPC 错误: {err}"));
    }
    let data_b64 = payload
        .pointer("/result/value/data/0")
        .and_then(Value::as_str)
        .ok_or_else(|| "Candy Machine 账户不存在或无法读取。".to_string())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_b64)
        .map_err(|err| format!("Base64 解码失败: {err}"))?;
    if bytes.len() < ITEMS_REDEEMED_OFFSET + 8 {
        return Err("Candy Machine 账户数据过短。".to_string());
    }
    let mut buf = [0_u8; 8];
    buf.copy_from_slice(&bytes[ITEMS_REDEEMED_OFFSET..ITEMS_REDEEMED_OFFSET + 8]);
    Ok(u64::from_le_bytes(buf))
}
