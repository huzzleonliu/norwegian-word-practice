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
│  │  └─ public/
│  └─ solana-gate/            # Solana / 门禁共享逻辑（占位）
└─ candy_machine/             # 独立目录，不进 workspace
```

---

## 2. 本地开发（在 crate 内）

```bash
cd crates/nwp-web
npm ci
trunk serve
```

说明：

- `Trunk.toml` / `tailwind.config.js` / `package.json` 都跟 web crate 走。
- 根目录不再放前端开发入口配置。
- 若环境变量 `NO_COLOR=1` 导致 trunk 报参错，使用 `NO_COLOR=true trunk serve`。

依赖其他 workspace crate 时，在 `crates/nwp-web/Cargo.toml` 写 path 依赖即可；在 crate 内跑 `trunk serve` 仍可正常解析。

---

## 3. 构建与测试

根目录：

```bash
cargo check --workspace --all-targets
cargo test
```

web crate：

```bash
cd crates/nwp-web
NO_COLOR=true trunk build --release
```

---

## 4. Docker 打包链路

根目录 `Dockerfile`：

1. builder：安装 wasm target / trunk / node / tailwind
2. 拷贝 workspace + `crates/nwp-web` + `crates/solana-gate`
3. 在 `crates/nwp-web` 内 `npm ci` + `trunk build --release`
4. runner：nginx 托管 `dist/`，使用 `crates/nwp-web/nginx.conf`

```bash
docker build -t norwegian-word-practice:latest .
docker run --rm -p 8080:80 norwegian-word-practice:latest
```

---

## 5. 注意事项

- `Cargo.lock` 只保留在仓库根目录。
- `candy_machine/` 保持独立，不加入 workspace。
- 新增共享库时优先放进 `crates/`，再由 `nwp-web` 通过 path 依赖引用。
