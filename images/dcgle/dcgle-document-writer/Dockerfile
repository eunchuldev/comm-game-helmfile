FROM ekidd/rust-musl-builder:stable as builder

ADD --chown=rust:rust . ./

RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/dcgle-document-writer \
    /usr/local/bin/
