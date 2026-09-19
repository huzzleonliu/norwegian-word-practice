# 架构说明

## 技术栈

| 层 | 选型 |
| --- | --- |
| 语言 / 运行时 | Rust → `wasm32-unknown-unknown` |
| UI | [Leptos](https://leptos.dev/) 0.8（CSR）+ `leptos_router` |
| 打包 | [Trunk](https://trunkrs.dev/) |
| 样式 | Tailwind CSS 3（通过 npm / CLI） |
| 序列化 | serde / serde_json / csv |
| 词库保护 | 自定义加密信封（`.nwpdict`，见 `dictionary_crypto`） |
| 部署 | 多阶段 Docker：builder 产出 `dist/`，nginx 托管静态资源 |

开发入口在 **crate 内**，不在仓库根：

```bash
cd crates/nwp-web && npm ci && trunk serve
```

## Crate 职责

### `nwp-web`

主应用。大致分层：

```text
src/
├─ main.rs              # 路由、全局 context、语言前缀布局
├─ app_state.rs         # Lexicon / Practice / UI / 导航回调
├─ pages/               # 页面组件（与 AppPage 一一对应）
├─ components/          # 可复用 UI（练习引擎、词库浏览器、查词等）
├─ structures/          # WordBankEntry、练习结果、字段元数据
├─ utils/               # CSV、校验、加密、i18n、文件加载
├─ crypto_module/       # 与加密相关的常量等
└─ tests/               # 随 bin 编译的单元测试
```

**练习引擎**（`components/practice_engine/`）把「设置 / 题目渲染 / 判题纯函数 / 流程按钮」拆开，词库练习与多数系列练习共用。

**系列练习**（`pages/serise_practice/`）在共享初始化逻辑上各自实现题型（例如数词分块、国家按 tag 分块）。

### `solana-gate`

门禁运行时：意图、钱包状态、NFT 持有校验结果，以及「需要 NFT 才能进入」的页面/按钮守卫。

### `solana-leptos-component`

钱包连接、Mint 页、Candy Machine / Guard 相关常量与查询、持仓判断等。与 `candy_machine/` 目录中的链上配置相互配合，但该目录本身不进入 Cargo workspace。

## 路由模型

每个逻辑页面是一个 `AppPage`（见 `pages/mod.rs`）。对外 URL 带语言前缀：

| 逻辑路径 | 说明 |
| --- | --- |
| `/` | 首页 |
| `/practice` | 练习模式选择 |
| `/lexicon` | 词库选题 |
| `/lexicon/practice` | 打字/拼写练习 |
| `/lexicon/remember` | 记忆对照 |
| `/lexicon/summary` | 练习总结 |
| `/editor` | 本地词库编辑 |
| `/series` | 系列选择 |
| `/series/number` 等 | 各系列练习页 |
| `/player` | 跟读播放器 |

完整地址形如 `/ch/series/country`、`/en/lexicon/practice`。无前缀的旧路径会重定向到默认中文前缀。

编程式导航通过 `NavigateToPage` context，内部走 `leptos_router`，对外仍是 `set(AppPage)`。

## 关键数据流

1. **选题**：练习模式页或系列选择页加载 `.nwpdict` → 写入 `LexiconState.entries`，并把可练 id 写入 `PracticeState.selected_word_entry_ids`。
2. **作答**：题目组件用 `{entry_id}::{field}` 作为输入键；检查时走 `evaluate_check_answers` 等纯函数。
3. **统计**：本轮写入 `temp_practice_result`；完成练习时合并进累计 `practice_result`。
4. **编辑器提交**：脏行经校验与 id 重算后合并回全局词库；导出时序列化为加密 `.nwpdict`。

内置词库通过相对 URL（如 `/data/series-word-bank/series-word-bank.nwpdict`）由 Trunk 静态托管；加载时带 cache-busting 查询参数。

## 应用内帮助

`crates/nwp-web/docs/help/*.md` 由 `markdown_view!` 在编译期嵌入。修改帮助文档后需触发 Trunk 重建（`Trunk.toml` 已 watch `docs/help`）。
