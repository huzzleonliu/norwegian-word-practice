# 开发指南

## 环境要求

- Rust（建议与 Docker 中一致的较新稳定版；项目 `edition = "2024"`）
- `wasm32-unknown-unknown`：`rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/)（Docker 锁定示例版本见根 `Dockerfile`）
- Node.js + npm（Tailwind 与前端工具链）
- 可选：Docker（发布镜像）

## 本地运行

```bash
cd crates/nwp-web
npm ci
trunk serve
```

若环境变量 `NO_COLOR=1` 导致 Trunk 参数解析异常，请使用：

```bash
NO_COLOR=true trunk serve
```

浏览器访问 Trunk 提示的本地地址（通常为 `http://127.0.0.1:8080`）。

`Trunk.toml` / `tailwind.config.js` / `package.json` 均位于 `crates/nwp-web/`，不要在仓库根目录寻找前端入口。

## 检查与测试

在仓库根目录：

```bash
cargo check --workspace --all-targets
cargo test
```

`nwp-web` 以 bin 形式承载多数单元测试（`src/tests/` 与页面/组件内 `#[cfg(test)]`）。

发布构建：

```bash
cd crates/nwp-web
NO_COLOR=true trunk build --release
```

产物在 `crates/nwp-web/dist/`。

## Docker

在仓库根目录：

```bash
docker build -t norwegian-word-practice:latest .
docker run --rm -p 8080:80 norwegian-word-practice:latest
```

然后访问 `http://localhost:8080`。

镜像流程概要：安装 wasm target / Trunk / Node → 在 `crates/nwp-web` 内 `npm ci` + `trunk build --release` → nginx 托管 `dist/`，配置为 `crates/nwp-web/nginx.conf`。

## Workspace 约定

- **只保留根目录一份 `Cargo.lock`。**
- 新增共享库优先放进 `crates/`，由 `nwp-web` 用 path 依赖引用。
- `candy_machine/` 保持独立，**不加入** workspace；默认在 `.gitignore` 中。
- `TEMP_assets/`、明文 CSV 模板等为本地临时资产，勿提交密钥或未脱敏数据。
- 修改 `docs/help/**` 会触发 Trunk 监视重建；应用内帮助与仓库 `documents/` 职责不同，勿混放。

## 常用改动入口

| 需求 | 优先查看 |
| --- | --- |
| 新页面 / 路由 | `pages/mod.rs`、`main.rs`、`docs/help/` |
| 练习判题 / 回答预设 | `components/practice_engine/` |
| 新系列练习 | `pages/serise_practice/`、`serise_select.rs`、系列词库 tag |
| 词库字段 / CSV | `structures/`、`utils/csv_schema.rs`、`field_meta` |
| 加密导入导出 | `utils/dictionary_crypto.rs`、`lexicon_file.rs` |
| 门禁 / Mint | `solana-gate`、`solana-leptos-component` |

更完整的贡献流程见根目录 [CONTRIBUTING.md](../CONTRIBUTING.md)。
