FROM rust:slim as builder

WORKDIR /usr/src

RUN USER=root cargo new autodok
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock /usr/src/autodok/
WORKDIR /usr/src/autodok
RUN cargo build --target x86_64-unknown-linux-musl --release

COPY src /usr/src/autodok/src/
RUN touch /usr/src/autodok/src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release


FROM alpine:3 AS runtime
RUN apk add curl --no-cache
COPY --from=builder /usr/src/autodok/target/x86_64-unknown-linux-musl/release/autodok /usr/local/bin
EXPOSE 3000
CMD ["/usr/local/bin/autodok"]
