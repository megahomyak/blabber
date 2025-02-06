FROM alpine:3.21.2 AS build
RUN apk add --no-cache curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target x86_64-unknown-linux-musl
ENV PATH=/root/.cargo/bin:$PATH
RUN echo $PATH
COPY Cargo.lock Cargo.toml .
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release --target=x86_64-unknown-linux-musl
WORKDIR /app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /app/target/release/server /app
CMD ["/app"]
