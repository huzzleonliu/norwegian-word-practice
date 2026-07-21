use leptos::prelude::*;
use solana_gate::{
    CryptoGateMode, GateIntent, GateWalletState, NftGateState, OwnershipStatus,
};
use solana_leptos_component::WalletState;

/// `nwp-web` 对两个 Solana 库的共享信号封装。
#[derive(Clone, Copy)]
pub struct CryptoSignals {
    pub wallet_address: ReadSignal<Option<String>>,
    pub set_wallet_address: WriteSignal<Option<String>>,
    pub wallet_busy: ReadSignal<bool>,
    pub set_wallet_busy: WriteSignal<bool>,
    pub wallet_status: ReadSignal<String>,
    pub set_wallet_status: WriteSignal<String>,
    pub wallet_connect_nonce: ReadSignal<u64>,
    pub set_wallet_connect_nonce: WriteSignal<u64>,
    pub gate_mode: ReadSignal<CryptoGateMode>,
    pub gate_ownership: ReadSignal<OwnershipStatus>,
    pub set_gate_ownership: WriteSignal<OwnershipStatus>,
    pub gate_pending: ReadSignal<Option<GateIntent>>,
    pub set_gate_pending: WriteSignal<Option<GateIntent>>,
}

impl CryptoSignals {
    pub fn wallet_state(self) -> WalletState {
        WalletState {
            address: self.wallet_address,
            set_address: self.set_wallet_address,
            busy: self.wallet_busy,
            set_busy: self.set_wallet_busy,
            status: self.wallet_status,
            set_status: self.set_wallet_status,
            connect_nonce: self.wallet_connect_nonce,
            set_connect_nonce: self.set_wallet_connect_nonce,
        }
    }

    pub fn gate_wallet_state(self) -> GateWalletState {
        GateWalletState {
            address: self.wallet_address,
            set_connect_nonce: self.set_wallet_connect_nonce,
        }
    }

    pub fn gate_state(self) -> NftGateState {
        NftGateState {
            mode: self.gate_mode,
            ownership: self.gate_ownership,
            set_ownership: self.set_gate_ownership,
            pending: self.gate_pending,
            set_pending: self.set_gate_pending,
        }
    }
}

/// 根据构建 feature 解析当前默认门禁模式。
pub fn gate_mode_from_features() -> CryptoGateMode {
    if cfg!(feature = "no-blockchain-mode") {
        CryptoGateMode::NoBlockchain
    } else {
        CryptoGateMode::CryptoGate
    }
}

/// 初始化 Solana 相关共享信号。
pub fn create_crypto_signals(initial_gate_mode: CryptoGateMode) -> CryptoSignals {
    let (wallet_address, set_wallet_address) = signal(Option::<String>::None);
    let (wallet_busy, set_wallet_busy) = signal(false);
    let (wallet_status, set_wallet_status) = signal(String::new());
    let (wallet_connect_nonce, set_wallet_connect_nonce) = signal(0_u64);
    let (gate_mode, _set_gate_mode) = signal(initial_gate_mode);
    let (gate_ownership, set_gate_ownership) = signal(OwnershipStatus::Disconnected);
    let (gate_pending, set_gate_pending) = signal(Option::<GateIntent>::None);

    CryptoSignals {
        wallet_address,
        set_wallet_address,
        wallet_busy,
        set_wallet_busy,
        wallet_status,
        set_wallet_status,
        wallet_connect_nonce,
        set_wallet_connect_nonce,
        gate_mode,
        gate_ownership,
        set_gate_ownership,
        gate_pending,
        set_gate_pending,
    }
}
