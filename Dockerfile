FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin symposium
WORKDIR ./symposium
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY . .

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/symposium*
RUN cargo build --release

FROM alpine:latest

WORKDIR /app

COPY --from=builder /home/rust/src/symposium/target/x86_64-unknown-linux-musl/release/symposium .

COPY ./static ./static

CMD ["./symposium", "0.0.0.0", "8080"]
