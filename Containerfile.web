FROM docker.io/library/rust:1.90-slim AS builder

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN apt-get update && apt-get install -y nodejs npm && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock index.html Trunk.toml ./
COPY src ./src
COPY data ./data
COPY style ./style
COPY package.json package-lock.json ./
COPY tailwind.config.js ./

RUN npm ci
RUN trunk build --release

FROM docker.io/library/nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
