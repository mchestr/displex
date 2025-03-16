FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN update-ca-certificates
WORKDIR /app

FROM node:20-alpine AS web-builder
WORKDIR /app
COPY web/package.json .
RUN npm install
COPY web .
RUN npm run build

FROM chef AS planner
COPY server .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS app-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --profile dist --recipe-path recipe.json
COPY server .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --profile dist --bin displex --target x86_64-unknown-linux-musl

# taken from https://medium.com/@lizrice/non-privileged-containers-based-on-the-scratch-image-a80105d6d341
FROM ubuntu:latest AS user-creator
RUN groupadd -g 1001 displex \
        && useradd -u 1001 -g 1001 displex \
        && mkdir /data \
        && chown -R displex:displex /data

FROM scratch AS runtime
COPY --from=user-creator /etc/passwd /etc/passwd
COPY --from=user-creator --chown=displex:displex /data /data

VOLUME [ "/data" ]
WORKDIR /data

USER displex
ENV RUST_LOG="displex=info,sea_orm=info" \
    DISPLEX_HTTP__HOST=0.0.0.0 \
    DISPLEX_HTTP__PORT=8080 \
    DATABASE_URL=sqlite://displex.db?mode=rwc
COPY --from=app-builder --chown=displex:displex /app/target/x86_64-unknown-linux-musl/dist/displex /displex
COPY --from=web-builder --chown=displex:displex /app/dist /dist

ENTRYPOINT ["/displex"]
