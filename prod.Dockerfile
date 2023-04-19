FROM codenames-rs-builder AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin back_end --target x86_64-unknown-linux-musl \
    && trunk build --release

FROM alpine
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/back_end /usr/local/bin/app
COPY --from=builder /build/front_end/dist /app/dist
ENV FRONT_END_DIR /app/dist
CMD ["app"]
