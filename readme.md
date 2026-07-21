# Norwegian Word Practice

一个基于 Rust + Leptos (WASM) 的挪威语词汇练习工具，支持：

- 词库练习（可筛选、可配置字段）
- 系列练习（数词、月份、代词、疑问词）
- 本地词库编辑（单条/多条新增、表格编辑、导入导出）
- 练习结果导入导出

## Workspace 结构

```text
.
├─ Cargo.toml / Cargo.lock / Dockerfile / readme*
└─ crates/
   ├─ nwp-web/          # 网站 app（Leptos + Trunk）
   └─ solana-gate/      # Solana / 门禁共享逻辑（占位）
```

## 本地开发

在 web crate 内执行：

```bash
cd crates/nwp-web
npm ci
trunk serve
```

> 如果你的环境里设置了 `NO_COLOR=1`，请改成：
>
> ```bash
> NO_COLOR=true trunk serve
> ```

## 构建与测试

在仓库根目录：

```bash
cargo check --workspace --all-targets
cargo test
```

在 web crate 内：

```bash
cd crates/nwp-web
NO_COLOR=true trunk build --release
```

## Docker 打包

在仓库根目录执行：

```bash
docker build -t norwegian-word-practice:latest .
docker run --rm -p 8080:80 norwegian-word-practice:latest
```

然后访问 `http://localhost:8080`。
