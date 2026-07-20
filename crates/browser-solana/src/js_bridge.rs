//! Low-level helpers for calling functions attached to `window`.

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::error::BrowserSolanaError;

/// Best-effort extraction of an error message from a JS rejection value.
pub fn js_error_message(err: JsValue) -> String {
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
    "JavaScript bridge call failed.".to_string()
}

fn window_fn(global_name: &str) -> Result<Function, BrowserSolanaError> {
    let window = web_sys::window().ok_or_else(|| BrowserSolanaError::new("Browser window unavailable."))?;
    let value = Reflect::get(&window, &JsValue::from_str(global_name)).map_err(|_| {
        BrowserSolanaError::new(format!(
            "Window function `{global_name}` is not available. Did the page script load?"
        ))
    })?;
    if value.is_undefined() || value.is_null() {
        return Err(BrowserSolanaError::new(format!(
            "Window function `{global_name}` is not loaded yet. Refresh and retry."
        )));
    }
    value
        .dyn_ref::<Function>()
        .cloned()
        .ok_or_else(|| BrowserSolanaError::new(format!("`{global_name}` is not a function.")))
}

/// Call `window[global_name](...args)` and await the returned Promise.
pub async fn call_window_async(
    global_name: &str,
    args: &[JsValue],
) -> Result<JsValue, BrowserSolanaError> {
    let func = window_fn(global_name)?;
    let this = JsValue::NULL;
    let result = match args.len() {
        0 => func.call0(&this),
        1 => func.call1(&this, &args[0]),
        2 => func.call2(&this, &args[0], &args[1]),
        3 => func.call3(&this, &args[0], &args[1], &args[2]),
        _ => {
            let array = js_sys::Array::new();
            for arg in args {
                array.push(arg);
            }
            func.apply(&this, &array)
        }
    }
    .map_err(|err| BrowserSolanaError::new(js_error_message(err)))?;

    JsFuture::from(Promise::resolve(&result))
        .await
        .map_err(|err| BrowserSolanaError::new(js_error_message(err)))
}
