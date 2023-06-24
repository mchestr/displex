FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update && apt-get install -y musl-tools musl-dev
RUN update-ca-certificates
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS app-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --profile dist --recipe-path recipe.json
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --profile dist --bin displex --target x86_64-unknown-linux-musl

# taken from https://medium.com/@lizrice/non-privileged-containers-based-on-the-scratch-image-a80105d6d341
FROM ubuntu:latest as user-creator
RUN groupadd -g 1001 displex \
        && useradd -u 1001 -g 1001 displex

FROM scratch AS runtime
COPY --from=user-creator /etc/passwd /etc/passwd
USER displex

WORKDIR /data
RUN chown -R displex:displex /data
ENV RUST_LOG="displex=info,sea_orm=info" \
    DISPLEX_HTTP__HOST=0.0.0.0 \
    DISPLEX_HTTP__PORT=8080 \
    DATABASE_URL=sqlite://displex.db?mode=rwc
COPY --from=app-builder --chown=displex:displex /app/target/x86_64-unknown-linux-musl/dist/displex /app
ENTRYPOINT ["/app"]
