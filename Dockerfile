FROM rust:1.41.0-slim as builder

WORKDIR /opt/thumbnailer_rust
COPY . /opt/thumbnailer_rust
RUN cargo install --path .

FROM debian:stable-slim

COPY --from=builder /usr/local/cargo/bin/thumbnailer_rust /app/thumbnailer_rust
WORKDIR /app

CMD [ "./thumbnailer_rust" ]