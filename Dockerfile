FROM lukemathwalker/cargo-chef:0.1.59-rust-1.69.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && apt-get install -y libpq-dev
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin displex

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
RUN apt-get update && apt-get install -y libpq-dev ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/displex /usr/local/bin
ENTRYPOINT ["/usr/local/bin/displex"]