//! NFT 门禁状态：持仓资格与连接成功后的待办意图。

use leptos::prelude::*;

use crate::pages::AppPage;

/// 当前钱包相对 OAOA Collection 的持仓状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnershipStatus {
    /// 未连接钱包。
    Disconnected,
    /// 正在查询链上持仓。
    Checking,
    /// 持有本系列至少一枚 NFT。
    Owns,
    /// 已连接但不持有。
    Missing,
}

/// 钱包连接完成后需要继续执行的导航意图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateIntent {
    /// 进入需持有 NFT 的页面（同标签）。
    EnterPage(AppPage),
    /// 打开 Mint 页（同标签）。
    OpenMint,
}

/// 全局 NFT 门禁状态，经 Context 注入。
#[derive(Clone, Copy)]
pub struct NftGateState {
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
