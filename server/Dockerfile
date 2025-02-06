FROM rust:1.84.1-alpine3.21 AS build
RUN apk add --no-cache musl-dev
COPY Cargo.lock Cargo.toml .
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl
WORKDIR /app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/server /app
CMD ["/app"]
