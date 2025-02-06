FROM rust:1.84.1-alpine3.21 AS build
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.lock Cargo.toml .
RUN mkdir src && echo 'fn main(){println!("I shouldnt be in the final build!")}' > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN rm /app/target/x86_64-unknown-linux-musl/release/server
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/server /app
CMD ["/app"]
