FROM trunk
WORKDIR /app
COPY . .
RUN cargo build --release --bin back_end \
    && trunk build --release

FROM debian:bullseye-slim
COPY --from=0 /app/target/release/back_end /app/app
COPY --from=0 /app/front_end/dist /app/dist
ENV FRONT_END_DIR /app/dist
CMD ["/app/app"]
