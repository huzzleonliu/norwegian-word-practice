use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

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
    "Ownership check failed.".to_string()
}

/// 返回 `owner` 是否持有本系列（Collection）至少一枚 NFT。
pub async fn wallet_owns_collection_nft(owner: &str) -> Result<bool, String> {
    let window = web_sys::window().ok_or_else(|| "浏览器环境不可用。".to_string())?;
    let check_fn = Reflect::get(&window, &JsValue::from_str("__oaoaHasCollectionNft"))
        .map_err(|_| "未找到持仓校验脚本，请刷新页面。".to_string())?;
    if check_fn.is_undefined() || check_fn.is_null() {
        return Err("持仓校验脚本尚未加载，请刷新页面后重试。".to_string());
    }
    let check_fn = check_fn
        .dyn_ref::<Function>()
        .ok_or_else(|| "持仓校验脚本接口无效。".to_string())?;
    let promise = check_fn
        .call1(&JsValue::NULL, &JsValue::from_str(owner))
        .map_err(js_error_message)?;
    let value = JsFuture::from(Promise::resolve(&promise))
        .await
        .map_err(js_error_message)?;
    value
        .as_bool()
        .ok_or_else(|| "持仓校验未返回布尔结果。".to_string())
}
