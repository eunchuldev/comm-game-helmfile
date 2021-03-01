FROM ekidd/rust-musl-builder:stable as builder

USER root

RUN curl -fsSL https://deb.nodesource.com/setup_15.x | sudo -E bash - && \
  sudo apt-get -y install nodejs

USER rust

ADD --chown=rust:rust . ./

RUN cargo build --release

FROM alpine:latest

RUN apk --no-cache add ca-certificates

COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/server \
    /usr/local/bin/

COPY --from=builder \
    /home/rust/src/svelte-app/public \
    svelte-app/public

COPY --from=builder \
    /home/rust/src/migrations \
    migrations

ENTRYPOINT server
