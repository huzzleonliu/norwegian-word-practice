use leptos::prelude::*;

/// 门禁运行模式：
/// - `CryptoGate`：启用钱包连接 + NFT 持仓校验。
/// - `NoBlockchain`：完全绕过区块链门禁（未来可切换使用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptoGateMode {
    CryptoGate,
    NoBlockchain,
}

/// 当前钱包相对 Collection 的持仓状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnershipStatus {
    Disconnected,
    Checking,
    Owns,
    Missing,
}

/// 连接/校验完成后继续执行的待办意图。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateIntent {
    EnterPath(String),
    OpenMint,
}

/// 门禁动作运行时：由业务应用注入导航/刷新等能力。
#[derive(Clone)]
pub struct GateRuntime {
    pub navigate_to_path: Callback<String>,
    pub open_mint_in_new_tab: Callback<()>,
    pub request_ownership_refresh: Callback<String>,
    pub mint_path: String,
    pub fallback_path: String,
}

/// 门禁使用的钱包桥接状态（与具体钱包组件实现解耦）。
#[derive(Clone, Copy)]
pub struct GateWalletState {
    pub address: ReadSignal<Option<String>>,
    pub set_connect_nonce: WriteSignal<u64>,
}

impl GateWalletState {
    pub fn is_connected(self) -> bool {
        self.address.get_untracked().is_some()
    }

    pub fn request_connect(self) {
        self.set_connect_nonce.update(|n| *n = n.wrapping_add(1));
    }
}

/// 全局 NFT 门禁状态，经 Context 注入。
#[derive(Clone, Copy)]
pub struct NftGateState {
    pub mode: ReadSignal<CryptoGateMode>,
    pub ownership: ReadSignal<OwnershipStatus>,
    pub set_ownership: WriteSignal<OwnershipStatus>,
    pub pending: ReadSignal<Option<GateIntent>>,
    pub set_pending: WriteSignal<Option<GateIntent>>,
}

impl NftGateState {
    pub fn clear_pending(self) {
        self.set_pending.set(None);
    }
}
