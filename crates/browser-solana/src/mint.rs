//! Mint helpers via a page-injected `window` function.
//!
//! Expected JS shape:
//! ```js
//! window.myMintNft = async () => ({ signature: string, mint?: string })
//! ```

use js_sys::Reflect;
use wasm_bindgen::JsValue;

use crate::error::BrowserSolanaError;
use crate::js_bridge::call_window_async;

/// Result of a successful mint bridge call.
#[derive(Debug, Clone)]
pub struct MintResult {
    /// Transaction signature.
    pub signature: String,
    /// Minted asset address when the bridge provides it.
    pub mint: String,
}

/// Call `window[global_fn]()` and parse `{ signature, mint }` from the result.
pub async fn mint_via_window_fn(global_fn: &str) -> Result<MintResult, BrowserSolanaError> {
    let value = call_window_async(global_fn, &[]).await?;
    let signature = Reflect::get(&value, &JsValue::from_str("signature"))
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| BrowserSolanaError::new("Mint succeeded but signature was missing."))?;
    let mint = Reflect::get(&value, &JsValue::from_str("mint"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    Ok(MintResult { signature, mint })
}
