# 贡献指引（Contributing）

感谢你对 **Norwegian Word Practice** 的兴趣。本文说明如何有效地参与开发。

## 开始之前

1. 阅读 [documents/overview.md](documents/overview.md) 与 [documents/development.md](documents/development.md)。
2. 能本地跑通：`cd crates/nwp-web && npm ci && trunk serve`。
3. 确认改动范围：UI / 练习引擎 / 词库格式 / Solana 门禁，尽量单一职责。

## 行为准则（简要）

- 对学习者和贡献者保持友善、具体、可操作的反馈。
- 不提交密钥、私钥、未脱敏的个人词库或 API Key。
- 讨论功能时优先说明「谁在什么场景下受益」，避免无上下文的大范围重写。

## 开发流程

1. Fork（或在有权限的分支上）拉取最新 `main`。
2. 创建主题分支，例如 `feat/country-help`、`fix/practice-null-field`。
3. 实现改动；保持与现有代码风格一致（模块注释、Leptos 组件模式、中英 `tr(...)` 文案成对出现）。
4. 在根目录运行：
   ```bash
   cargo check --workspace --all-targets
   cargo test
   ```
5. 若改动 UI，用 `trunk serve` 做一次手动点测（相关语言前缀路径都看一眼更佳）。
6. 提交信息写清**为什么**（可用中文或英文）；参考仓库既有风格。
7. 打开 Pull Request：说明动机、改动摘要、测试方式；关联 Issue（如有）。

## 提交与 PR 建议

**适合合并的改动**

- Bug 修复、无障碍改进、文档与帮助文案
- 小范围新系列 / 新字段，并带上 `docs/help` 与必要测试
- 性能或可访问性改进，且不破坏既有练习数据假设

**请先开 Issue 讨论的改动**

- 词库 schema / 加密格式不兼容变更
- 大规模 UI 重设计或依赖大版本升级
- 新的链上经济/门禁策略

**不要做的事**

- 把 `TEMP_assets/`、密钥、真实用户词库打进仓库
- 用 `--force` 改写已共享历史（除非维护者明确要求）
- 在无关 PR 里顺手大规模格式化全仓库

## 文档与帮助

| 位置 | 用途 |
| --- | --- |
| `documents/` | 仓库级架构与开发说明 |
| `crates/nwp-web/docs/help/` | 应用内帮助（改完需 Trunk 能重建） |
| `README.md` | 对外快速入口；结构性变化时请同步更新 |

## License 与贡献授权

贡献默认在 **Apache-2.0** 下提供（与项目相同），除非你另行明确声明。详见根目录 [LICENSE](LICENSE)。

## 问题反馈

- GitHub Issues：缺陷、需求、安全疑虑（勿在公开 Issue 贴私钥）
- 应用内「问题反馈」：可前往 Discord 与作者讨论产品与 OAOA 相关话题

再次感谢你的时间与补丁。
