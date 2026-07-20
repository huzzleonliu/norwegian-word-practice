FROM docker.io/library/rust:1.90-slim AS builder

WORKDIR /workspace

RUN rustup target add wasm32-unknown-unknown \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && npm install --global tailwindcss@3.4.17 \
    && cargo install trunk --version 0.21.14 --locked \
    && rm -rf /var/lib/apt/lists/*

# Workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/nwp-web/Cargo.toml crates/nwp-web/build.rs crates/nwp-web/index.html crates/nwp-web/Trunk.toml ./crates/nwp-web/
COPY crates/browser-solana/Cargo.toml ./crates/browser-solana/
COPY crates/browser-solana/src ./crates/browser-solana/src
COPY crates/leptos-solana-gate/Cargo.toml ./crates/leptos-solana-gate/
COPY crates/leptos-solana-gate/src ./crates/leptos-solana-gate/src

# Frontend toolchain files
COPY crates/nwp-web/package.json crates/nwp-web/package-lock.json crates/nwp-web/tailwind.config.js ./crates/nwp-web/

# App sources/assets
COPY crates/nwp-web/src ./crates/nwp-web/src
COPY crates/nwp-web/data ./crates/nwp-web/data
COPY crates/nwp-web/docs ./crates/nwp-web/docs
COPY crates/nwp-web/public ./crates/nwp-web/public
COPY crates/nwp-web/js ./crates/nwp-web/js
COPY crates/nwp-web/style ./crates/nwp-web/style

RUN npm --prefix crates/nwp-web ci
WORKDIR /workspace/crates/nwp-web
RUN NO_COLOR=true trunk build --release

FROM docker.io/library/nginx:alpine AS runner

COPY --from=builder /workspace/crates/nwp-web/dist /usr/share/nginx/html
COPY crates/nwp-web/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
