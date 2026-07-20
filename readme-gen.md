# Norwegian Word Practice 维护文档（readme-gen）

本文档面向维护者，描述当前 **workspace 结构**、构建链路和关键开发约束。

---

## 1. 当前项目结构

```text
.
├─ Cargo.toml                 # workspace 入口
├─ Cargo.lock                 # 只留根一份
├─ Dockerfile                 # 部署入口（根）
├─ readme.md / readme-gen.md
├─ crates/
│  ├─ nwp-web/                # 网站 app（Leptos/WASM）
│  │  ├─ Cargo.toml
│  │  ├─ Trunk.toml
│  │  ├─ index.html
│  │  ├─ package.json
│  │  ├─ tailwind.config.js
│  │  ├─ src/
│  │  ├─ style/
│  │  ├─ data/
│  │  ├─ docs/
│  │  ├─ public/
│  │  └─ nginx.conf
│  ├─ browser-solana/         # 通用浏览器 Solana WASM 适配
│  └─ leptos-solana-gate/     # Leptos 门禁状态机与组件
└─ candy_machine/             # 独立目录，不进 workspace
```

---

## 2. 本地开发（在 web crate 内）

### 2.1 依赖安装

```bash
cd crates/nwp-web
npm ci
```

### 2.2 启动开发服务器

```bash
cd crates/nwp-web
trunk serve
```

说明：

- `Trunk.toml`、`tailwind.config.js`、`package.json` 都在 `crates/nwp-web`。
- 若环境变量 `NO_COLOR=1` 导致 trunk 报参错，使用：

```bash
NO_COLOR=true trunk serve
```

---

## 3. 构建与测试

在 workspace 根目录：

```bash
cargo check --workspace --all-targets
cargo test --workspace
```

在 web crate 内：

```bash
cd crates/nwp-web
NO_COLOR=true trunk build --release
```

---

## 4. Docker 打包链路

根目录 `Dockerfile` 采用多阶段构建：

1. `rust:1.90-slim` 阶段：
   - 安装 `wasm32-unknown-unknown`
   - 安装 `trunk@0.21.14 --locked`
   - 安装 `nodejs/npm` 与全局 `tailwindcss`
   - 在 `crates/nwp-web` 内执行 `trunk build --release`
2. `nginx:alpine` 阶段：
   - 拷贝 `crates/nwp-web/dist/` 到 nginx 静态目录
   - 使用 `crates/nwp-web/nginx.conf`

构建：

```bash
docker build -t norwegian-word-practice:latest .
```

运行：

```bash
docker run --rm -p 8080:80 norwegian-word-practice:latest
```

---

## 5. 注意事项

- `Cargo.lock` 只保留根目录一份；成员 crate 内不要再放 lock 文件。
- 前端工具链（Trunk / Tailwind / npm）跟随 `nwp-web`，不要再放到仓库根目录。
- Solana 分层：
  - `browser-solana`：钱包 / JS bridge / Candy Machine RPC（无业务常量）
  - `leptos-solana-gate`：门禁状态机、`RequireNftPage`、`GatedNavigateButton`
  - `nwp-web`：OAOA 常量（`structures/nft_collection`）、`js/oaoa-mint.js`、i18n 包装与 Mint 页
- JS 助手 `crates/nwp-web/js/oaoa-mint.js` 由 Trunk 挂载；常量需与 `structures/nft_collection` 保持一致。
