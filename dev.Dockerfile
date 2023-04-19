FROM alpine
COPY ./target/x86_64-unknown-linux-musl/debug/back_end /usr/local/bin/app
CMD ["app"]
