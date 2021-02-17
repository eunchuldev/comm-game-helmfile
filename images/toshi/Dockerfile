FROM ekidd/rust-musl-builder:stable as builder

WORKDIR /home/rust

RUN git clone https://github.com/toshi-search/Toshi.git

WORKDIR /home/rust/Toshi

RUN cargo build --release



FROM alpine:latest
RUN apk --no-cache add ca-certificates curl
COPY --from=builder \
    /home/rust/Toshi/target/x86_64-unknown-linux-musl/release/toshi \
    /usr/local/bin/
COPY --from=builder \
    /home/rust/Toshi/config \
    /config

WORKDIR /

EXPOSE 8080

CMD ["toshi"]
