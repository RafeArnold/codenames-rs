FROM alpine
COPY ./target/x86_64-unknown-linux-musl/debug/back_end /usr/local/bin/app
COPY ./front_end/dist /app/dist
ENV FRONT_END_DIR /app/dist
CMD ["app"]
