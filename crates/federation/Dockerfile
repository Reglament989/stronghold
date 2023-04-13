FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install protobuf-compiler -y

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --package federation

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/federation /usr/local/bin
COPY config.toml /app/
EXPOSE 50051

ENTRYPOINT ["/usr/local/bin/federation"]