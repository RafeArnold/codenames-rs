FROM rust:slim
RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl\
    && cargo install --locked trunk
