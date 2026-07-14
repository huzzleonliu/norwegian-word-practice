//! 字符串列表打乱工具：用于随机化练习队列。

/// 原地打乱字符串切片（Fisher-Yates 变体）。
pub fn shuffle_strings(values: &mut [String]) {
    if values.len() <= 1 {
        return;
    }

    for idx in (1..values.len()).rev() {
        let swap_idx = random_index(idx + 1);
        values.swap(idx, swap_idx);
    }
}

/// 生成 `[0, upper_exclusive)` 范围内随机索引。
fn random_index(upper_exclusive: usize) -> usize {
    if upper_exclusive <= 1 {
        return 0;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let raw = js_sys::Math::random() * upper_exclusive as f64;
        let idx = raw.floor() as usize;
        idx.min(upper_exclusive - 1)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.subsec_nanos() as usize)
            .unwrap_or(0);
        nanos % upper_exclusive
    }
}
