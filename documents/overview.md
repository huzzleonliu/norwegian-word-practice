# 项目概览

## 定位

**Norwegian Word Practice（NWP）** 是一个在浏览器中运行的挪威语词汇练习工具，技术栈为 **Rust + Leptos（WASM）**。

它也是作者 Huzz 个人艺术创作系列 **Once and Once Again（OAOA）** 下的一个子项目：学习工具与链上/门禁相关能力可以共存，但核心价值仍是「可配置的挪威语拼写与变位练习」。

仓库：<https://github.com/huzzleonliu/norwegian-word-practice>

## 能做什么

- **词库练习**：从内置或导入的加密词库选题；可按中英、词性、标签筛选；可配置提示字段与回答字段（含基础/常用/全面预设）。
- **记忆对照**：同词库选题，侧重对照记忆而非判题流程。
- **系列练习**：固定主题——数词、月份、代词、疑问词、国家（按国家分块）。
- **本地词库编辑**：单条/批量添加、表格浏览与列配置、Ordbok / 翻译 / Gemini 辅助填空、导入导出 `.nwpdict`。
- **练习总结**：本轮与累计正确率、错题回顾；结果可导入导出。
- **国际化路由**：`/ch/...` 与 `/en/...` 前缀；界面语言与路径联动。
- **Solana / NFT 门禁（可选能力）**：钱包连接、Mint、持仓校验等，由独立 crate 承载，可与部分页面守卫结合。

## 仓库地图

```text
.
├─ Cargo.toml / Cargo.lock     # workspace
├─ Dockerfile                  # 生产镜像：trunk build → nginx
├─ README.md / CONTRIBUTING.md
├─ documents/                  # 本目录：维护与贡献文档
├─ candy_machine/              # OAOA 上链资产与 Sugar 相关（不进 workspace，默认 gitignore）
└─ crates/
   ├─ nwp-web/                 # 主站：Leptos CSR + Trunk + Tailwind
   ├─ solana-gate/             # NFT 门禁状态、守卫与导航组件
   └─ solana-leptos-component/ # 钱包、Mint、持仓查询等链上 UI/逻辑
```

内置词库数据在 `crates/nwp-web/data/`：

- `lexicon-word-bank/` — 通用练习词库（如 Byen er Bergen）
- `series-word-bank/series-word-bank.nwpdict` — 系列练习统一词库（按 tag 过滤）

## 相关文档

- 开发与部署：[development.md](./development.md)
- 架构：[architecture.md](./architecture.md)
- 功能与路径：[features.md](./features.md)
- 词库格式：[data-format.md](./data-format.md)
