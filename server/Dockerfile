FROM rust:1.84.1-alpine3.21 AS cargo-chef
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
WORKDIR /app

FROM cargo-chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM cargo-chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target=x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/server /app
CMD ["/app"]
