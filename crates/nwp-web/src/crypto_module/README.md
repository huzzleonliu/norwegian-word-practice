# crypto_module 归档说明（中文）

当前状态：`nwp-web` 已移除所有页面内正在生效的区块链功能（钱包按钮、Mint 页面展示、NFT 门禁守卫、相关路由逻辑）。

本目录用于**存档未来可恢复的接入方案**。当你需要重新引入链上能力时，可按下面步骤操作。

## 1) 恢复依赖

编辑 `crates/nwp-web/Cargo.toml`，在 `[dependencies]` 中加入：

- `solana-gate = { path = "../solana-gate" }`
- `solana-leptos-component = { path = "../solana-leptos-component" }`

如果未来想切换成 crates.io 版本，可将上面两项改为对应版本号依赖。

## 2) 恢复模块入口

在 `crates/nwp-web/src/main.rs` 顶部恢复：

- `mod crypto_module;`
- 对 `crypto_module::constants / locale / runtime / structures` 的 `use`
- 对 `solana-gate` 与 `solana-leptos-component` 的 `use`

## 3) 恢复全局上下文（Context）

在 `App()` 中：

1. 通过 `create_crypto_signals(gate_mode_from_features())` 创建信号
2. `provide_context` 注入：
   - `WalletState`
   - `GateWalletState`
   - `NftGateState`

这三类 Context 是组件库与门禁库协作的核心。

## 4) 恢复 AppChrome 协作逻辑

在 `AppChrome()` 中恢复：

1. 从 Context 读取 `GateWalletState` 与 `NftGateState`
2. 调用 `build_app_gate_runtime(...)`
3. 调用 `install_gate_effects(...)`
4. Mint 按钮调用 `request_mint_page(...)`

## 5) 恢复页面层 UI 与路由

### 顶部/底部控制区

- 顶部恢复 `<WalletConnectButton/>`
- 底部恢复 “Mint NFT” 按钮

### 路由

- `/editor` 改回门禁包装页（`RequireNftPage`）
- `/mint` 改回 `MintNftPage`（而不是重定向）

### 练习模式页

`crates/nwp-web/src/pages/practice_mode_select.rs` 中：

- 将本地词库按钮从普通 `<button>` 改回 `NftGatedNavigateButton`

## 6) 验证动作

恢复后建议依次执行：

1. `cargo check --workspace --all-targets --color=never`
2. `cargo test --workspace --no-run --color=never`
3. 手动验证：
   - 钱包连接/断开
   - `/mint` 页面打开与回退
   - `/editor` 门禁行为（未连接、未持有、持有）

## 7) 当前归档文件（可直接复用）

本目录现有 `constants.rs` / `locale.rs` / `runtime.rs` / `structures.rs` 已整理好协作桥接逻辑。  
重新启用时优先复用这些文件，避免重复开发。
