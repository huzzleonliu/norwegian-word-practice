//! Solana browser-extension wallet via `window.solana` (Phantom, Solflare, …).

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::error::BrowserSolanaError;
use crate::js_bridge::js_error_message;

fn window_solana_provider() -> Result<JsValue, BrowserSolanaError> {
    let window = web_sys::window().ok_or_else(|| BrowserSolanaError::new("Browser window unavailable."))?;
    let provider = Reflect::get(&window, &JsValue::from_str("solana"))
        .map_err(|_| BrowserSolanaError::new("Unable to read wallet provider."))?;
    if provider.is_undefined() || provider.is_null() {
        return Err(BrowserSolanaError::new(
            "No Solana wallet detected. Install Phantom / Solflare (or similar) and refresh.",
        ));
    }
    Ok(provider)
}

fn public_key_to_string(public_key: &JsValue) -> Result<String, BrowserSolanaError> {
    let to_string = Reflect::get(public_key, &JsValue::from_str("toString"))
        .map_err(|_| BrowserSolanaError::new("Unable to read public key."))?;
    let to_string = to_string
        .dyn_ref::<Function>()
        .ok_or_else(|| BrowserSolanaError::new("Invalid public key format."))?;
    let value = to_string
        .call0(public_key)
        .map_err(|_| BrowserSolanaError::new("Unable to stringify public key."))?;
    value
        .as_string()
        .ok_or_else(|| BrowserSolanaError::new("Public key is not a string."))
}

/// Request a connection to the injected Solana wallet; returns the base58 address.
pub async fn connect_wallet() -> Result<String, BrowserSolanaError> {
    let provider = window_solana_provider()?;
    let connect = Reflect::get(&provider, &JsValue::from_str("connect"))
        .map_err(|_| BrowserSolanaError::new("Wallet does not support connect."))?;
    let connect = connect
        .dyn_ref::<Function>()
        .ok_or_else(|| BrowserSolanaError::new("Invalid wallet connect interface."))?;
    let result = connect
        .call0(&provider)
        .map_err(|err| BrowserSolanaError::new(js_error_message(err)))?;
    let result = JsFuture::from(Promise::resolve(&result))
        .await
        .map_err(|err| BrowserSolanaError::new(js_error_message(err)))?;
    let public_key = Reflect::get(&result, &JsValue::from_str("publicKey"))
        .map_err(|_| BrowserSolanaError::new("Connect result missing publicKey."))?;
    if public_key.is_undefined() || public_key.is_null() {
        let public_key = Reflect::get(&provider, &JsValue::from_str("publicKey"))
            .map_err(|_| BrowserSolanaError::new("Unable to read wallet public key."))?;
        return public_key_to_string(&public_key);
    }
    public_key_to_string(&public_key)
}

/// Disconnect the current Solana wallet connection (no-op if unsupported).
pub async fn disconnect_wallet() -> Result<(), BrowserSolanaError> {
    let provider = window_solana_provider()?;
    let disconnect = Reflect::get(&provider, &JsValue::from_str("disconnect"))
        .map_err(|_| BrowserSolanaError::new("Wallet does not support disconnect."))?;
    if disconnect.is_undefined() || disconnect.is_null() {
        return Ok(());
    }
    let disconnect = disconnect
        .dyn_ref::<Function>()
        .ok_or_else(|| BrowserSolanaError::new("Invalid wallet disconnect interface."))?;
    let result = disconnect
        .call0(&provider)
        .map_err(|err| BrowserSolanaError::new(js_error_message(err)))?;
    if result.is_undefined() || result.is_null() {
        return Ok(());
    }
    let _ = JsFuture::from(Promise::resolve(&result))
        .await
        .map_err(|err| BrowserSolanaError::new(js_error_message(err)))?;
    Ok(())
}

/// Shorten a base58 address to `abcd…wxyz` for compact UI labels.
pub fn shorten_address(address: &str) -> String {
    let len = address.chars().count();
    if len <= 10 {
        return address.to_string();
    }
    let head: String = address.chars().take(4).collect();
    let tail: String = address.chars().skip(len.saturating_sub(4)).collect();
    format!("{head}…{tail}")
}
