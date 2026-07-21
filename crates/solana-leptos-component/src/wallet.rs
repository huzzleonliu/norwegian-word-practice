use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

fn window_solana_provider() -> Result<JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "浏览器环境不可用。".to_string())?;
    let provider = Reflect::get(&window, &JsValue::from_str("solana"))
        .map_err(|_| "无法读取钱包接口。".to_string())?;
    if provider.is_undefined() || provider.is_null() {
        return Err(
            "未检测到 Solana 钱包。请安装 Phantom / Solflare 等扩展后刷新页面。".to_string(),
        );
    }
    Ok(provider)
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
    "钱包操作失败。".to_string()
}

fn public_key_to_string(public_key: &JsValue) -> Result<String, String> {
    let to_string = Reflect::get(public_key, &JsValue::from_str("toString"))
        .map_err(|_| "无法读取公钥。".to_string())?;
    let to_string = to_string
        .dyn_ref::<Function>()
        .ok_or_else(|| "公钥格式无效。".to_string())?;
    let value = to_string
        .call0(public_key)
        .map_err(|_| "无法转换公钥。".to_string())?;
    value
        .as_string()
        .ok_or_else(|| "公钥不是字符串。".to_string())
}

/// 请求连接浏览器中的 Solana 钱包，返回公钥地址。
pub async fn connect_wallet() -> Result<String, String> {
    let provider = window_solana_provider()?;
    let connect = Reflect::get(&provider, &JsValue::from_str("connect"))
        .map_err(|_| "钱包不支持 connect。".to_string())?;
    let connect = connect
        .dyn_ref::<Function>()
        .ok_or_else(|| "钱包 connect 接口无效。".to_string())?;
    let result = connect.call0(&provider).map_err(js_error_message)?;
    let result = JsFuture::from(Promise::resolve(&result))
        .await
        .map_err(js_error_message)?;
    let public_key = Reflect::get(&result, &JsValue::from_str("publicKey"))
        .map_err(|_| "连接结果缺少 publicKey。".to_string())?;
    if public_key.is_undefined() || public_key.is_null() {
        let public_key = Reflect::get(&provider, &JsValue::from_str("publicKey"))
            .map_err(|_| "无法读取钱包公钥。".to_string())?;
        return public_key_to_string(&public_key);
    }
    public_key_to_string(&public_key)
}

/// 断开当前 Solana 钱包连接。
pub async fn disconnect_wallet() -> Result<(), String> {
    let provider = window_solana_provider()?;
    let disconnect = Reflect::get(&provider, &JsValue::from_str("disconnect"))
        .map_err(|_| "钱包不支持 disconnect。".to_string())?;
    if disconnect.is_undefined() || disconnect.is_null() {
        return Ok(());
    }
    let disconnect = disconnect
        .dyn_ref::<Function>()
        .ok_or_else(|| "钱包 disconnect 接口无效。".to_string())?;
    let result = disconnect.call0(&provider).map_err(js_error_message)?;
    if result.is_undefined() || result.is_null() {
        return Ok(());
    }
    let _ = JsFuture::from(Promise::resolve(&result))
        .await
        .map_err(js_error_message)?;
    Ok(())
}

/// 将公钥缩短为 `前4…后4` 便于按钮展示。
pub fn shorten_address(address: &str) -> String {
    let len = address.chars().count();
    if len <= 10 {
        return address.to_string();
    }
    let head: String = address.chars().take(4).collect();
    let tail: String = address.chars().skip(len.saturating_sub(4)).collect();
    format!("{head}…{tail}")
}
