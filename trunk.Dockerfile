FROM rust:slim
RUN rustup target add wasm32-unknown-unknown\
    && cargo install --locked trunk
