//! Ownership checks via a page-injected `window` function.
//!
//! Expected JS shape:
//! ```js
//! window.myHasCollectionNft = async (ownerPubkey) => boolean
//! ```

use wasm_bindgen::JsValue;

use crate::error::BrowserSolanaError;
use crate::js_bridge::call_window_async;

/// Call `window[global_fn](owner)` and interpret the result as `bool`.
pub async fn owns_via_window_fn(
    global_fn: &str,
    owner: &str,
) -> Result<bool, BrowserSolanaError> {
    let value = call_window_async(global_fn, &[JsValue::from_str(owner)]).await?;
    value
        .as_bool()
        .ok_or_else(|| BrowserSolanaError::new("Ownership check did not return a boolean."))
}
