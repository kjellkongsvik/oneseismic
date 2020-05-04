FROM rust
RUN apt-get update && apt-get install -y libzmq5-dev

WORKDIR /
RUN USER=rust cargo new --bin rapi
WORKDIR /rapi

COPY rapi/Cargo.lock ./Cargo.lock
COPY rapi/Cargo.toml ./Cargo.toml
COPY protos/ ../protos
COPY rapi/build.rs .

RUN cargo test
RUN rm -rf src/*

COPY rapi/src ./src

RUN rm -f ./target/debug/deps/rapi*
RUN rm -f ./target/release/deps/rapi*

RUN cargo test

RUN cargo build
CMD ["./target/debug/rapi"]
