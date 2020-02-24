FROM rust:1.41.0-buster as builder

RUN apt-get update -yqq && \
    apt-get upgrade -yqq

ARG VIPS_VERSION="8.8.1"
RUN apt-get install -yqq automake build-essential pkg-config libglib2.0-dev gobject-introspection gobject-introspection libxml2-dev libexpat1-dev libjpeg-dev libwebp-dev libpng-dev
RUN wget https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.gz
# Exit 0 added because warnings tend to exit the build at a non-zero status
RUN tar -xf vips-${VIPS_VERSION}.tar.gz && cd vips-${VIPS_VERSION} && ./configure && make && make install && ldconfig; exit 0

WORKDIR /opt/thumbnailer_rust
COPY . /opt/thumbnailer_rust
RUN cargo install --path .

FROM codechimpio/vips-alpine:8.8.1

COPY --from=builder /opt/thumbnailer_rust/target/release/deps /app/deps
COPY --from=builder /usr/local/cargo/bin/thumbnailer_rust /app/thumbnailer_rust
WORKDIR /app
CMD ["./thumbnailer_rust"]
EXPOSE 3000