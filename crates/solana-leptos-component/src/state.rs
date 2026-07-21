use leptos::prelude::*;

/// 钱包连接状态：供钱包按钮、Mint 页面与门禁逻辑共用。
#[derive(Clone, Copy)]
pub struct WalletState {
    pub address: ReadSignal<Option<String>>,
    pub set_address: WriteSignal<Option<String>>,
    pub busy: ReadSignal<bool>,
    pub set_busy: WriteSignal<bool>,
    pub status: ReadSignal<String>,
    pub set_status: WriteSignal<String>,
    /// 递增该计数可触发与“连接钱包”按钮相同的连接流程。
    pub connect_nonce: ReadSignal<u64>,
    pub set_connect_nonce: WriteSignal<u64>,
}

impl WalletState {
    pub fn is_connected(self) -> bool {
        self.address.get_untracked().is_some()
    }

    pub fn request_connect(self) {
        self.set_connect_nonce.update(|n| *n = n.wrapping_add(1));
    }
}
