//! Candy Machine account reads over Solana JSON-RPC.

use base64::Engine;
use gloo_net::http::Request;
use serde_json::Value;

use crate::error::BrowserSolanaError;

/// Parameters for reading `items_redeemed` from a Candy Machine account.
#[derive(Debug, Clone, Copy)]
pub struct CandyMachineQuery<'a> {
    /// JSON-RPC endpoint (e.g. `https://api.devnet.solana.com`).
    pub rpc_url: &'a str,
    /// Candy Machine account address (base58).
    pub candy_machine_id: &'a str,
    /// Byte offset of the `items_redeemed` u64 field in account data.
    ///
    /// For `mpl-candy-machine-core` this is commonly `112`.
    pub items_redeemed_offset: usize,
}

/// Fetch `items_redeemed` (sold count) for a Candy Machine.
pub async fn fetch_items_redeemed(query: CandyMachineQuery<'_>) -> Result<u64, BrowserSolanaError> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            query.candy_machine_id,
            { "encoding": "base64", "commitment": "confirmed" }
        ]
    });
    let response = Request::post(query.rpc_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|err| BrowserSolanaError::new(format!("Failed to build RPC request: {err}")))?
        .send()
        .await
        .map_err(|err| BrowserSolanaError::new(format!("RPC request failed: {err}")))?;
    let payload: Value = response
        .json()
        .await
        .map_err(|err| BrowserSolanaError::new(format!("Failed to parse RPC response: {err}")))?;
    if let Some(err) = payload.get("error") {
        return Err(BrowserSolanaError::new(format!("RPC error: {err}")));
    }
    let data_b64 = payload
        .pointer("/result/value/data/0")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            BrowserSolanaError::new("Candy Machine account missing or unreadable.")
        })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_b64)
        .map_err(|err| BrowserSolanaError::new(format!("Base64 decode failed: {err}")))?;
    if bytes.len() < query.items_redeemed_offset + 8 {
        return Err(BrowserSolanaError::new(
            "Candy Machine account data too short for items_redeemed.",
        ));
    }
    let mut buf = [0_u8; 8];
    buf.copy_from_slice(
        &bytes[query.items_redeemed_offset..query.items_redeemed_offset + 8],
    );
    Ok(u64::from_le_bytes(buf))
}
