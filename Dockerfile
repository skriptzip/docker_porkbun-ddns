FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Cache dependencies separately from the sources
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/porkbun-ddns* target/release/deps/porkbun_ddns*

COPY src ./src
# Force a rebuild: the copied sources may be older than the dummy build output
RUN touch src/main.rs && cargo build --release

# The scratch image has no shell, so prepare /data here
RUN mkdir /data && touch /data/last_ip.txt && chown -R 1001:1001 /data


FROM scratch

LABEL org.opencontainers.image.title="Porkbun DDNS" \
      org.opencontainers.image.description="Dynamic DNS updater for Porkbun domains" \
      org.opencontainers.image.version="1.0.0" \
      org.opencontainers.image.authors="skriptzip" \
      org.opencontainers.image.source="https://github.com/skriptzip/docker_porkbun-ddns" \
      org.opencontainers.image.licenses="MIT" \
      maintainer="skriptzip"

COPY --from=builder /app/target/release/porkbun-ddns /porkbun-ddns
COPY --from=builder --chown=1001:1001 /data /data

USER 1001:1001

ENTRYPOINT ["/porkbun-ddns"]
