FROM alpine:3.21.2 AS build
RUN apk add --no-cache cargo=1.83.0-r0 rustup==1.25.2-r5
RUN $HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl
COPY Cargo.lock Cargo.toml .
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl
WORKDIR /app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /app/target/release/server /app
CMD ["/app"]
