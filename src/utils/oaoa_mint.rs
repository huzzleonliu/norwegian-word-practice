//! 调用浏览器端 `window.__oaoaMintNft` 完成 Candy Machine 铸造。

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone)]
pub struct MintResult {
    pub signature: String,
    pub mint: String,
}

fn js_error_message(err: JsValue) -> String {
    if let Some(s) = err.as_string() {
        return s;
    }
    if let Some(obj) = err.dyn_ref::<js_sys::Object>() {
        if let Ok(message) = Reflect::get(obj, &JsValue::from_str("message")) {
            if let Some(s) = message.as_string() {
                return s;
            }
        }
    }
    "Mint failed.".to_string()
}

/// 触发页面加载的 JS mint 助手。
pub async fn mint_nft_with_wallet() -> Result<MintResult, String> {
    let window = web_sys::window().ok_or_else(|| "浏览器环境不可用。".to_string())?;
    let mint_fn = Reflect::get(&window, &JsValue::from_str("__oaoaMintNft"))
        .map_err(|_| "未找到 mint 脚本，请刷新页面。".to_string())?;
    if mint_fn.is_undefined() || mint_fn.is_null() {
        return Err("Mint 脚本尚未加载，请刷新页面后重试。".to_string());
    }
    let mint_fn = mint_fn
        .dyn_ref::<Function>()
        .ok_or_else(|| "Mint 脚本接口无效。".to_string())?;
    let promise = mint_fn
        .call0(&JsValue::NULL)
        .map_err(|err| js_error_message(err))?;
    let value = JsFuture::from(Promise::resolve(&promise))
        .await
        .map_err(|err| js_error_message(err))?;
    let signature = Reflect::get(&value, &JsValue::from_str("signature"))
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Mint 成功但未返回 signature。".to_string())?;
    let mint = Reflect::get(&value, &JsValue::from_str("mint"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    Ok(MintResult { signature, mint })
}
