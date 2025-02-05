FROM alpine:3.21.2 AS build
RUN apk add --no-cache cargo=1.83.0-r0
COPY Cargo.lock Cargo.toml .
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.21.2
COPY --from=build /app/target/release/server /app/server
CMD ["/app/server"]
